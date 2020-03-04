extern crate clap;
extern crate prgrs;
extern crate reqwest;

use clap::{crate_authors, crate_version, value_t, App, Arg};
use prgrs::{writeln, Length, Prgrs};
use reqwest::header::COOKIE;
use std::fs;

fn check_url(url: &str, max_days: u32) -> Result<Vec<bool>, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).header(COOKIE, "over18=1").send()?.text()?;

    let mut days = Vec::new();
    for d in 0..max_days + 1 {
        if d == 0 {
            days.push(
                res.contains(" hours ago</a>")
                    || res.contains(" hour ago</a>")
                    || res.contains(" minutes ago</a>"),
            );
        } else if d == 1 {
            days.push(res.contains(">1 day ago</a>"));
        } else {
            days.push(res.contains(format!(">{} days ago</a>", d).as_str()));
        }
    }
    Ok(days)
}

fn post_in_last_n_days(url: &str, n: u32) -> bool {
    match check_url(url, n) {
        Err(e) => {
            writeln(&format!("Error: {}", e)).ok();
            false
        }
        Ok(days) => days.contains(&true),
    }
}

fn get_urls_with_recent_posts(urls: &[String], num_days: u32) -> Vec<&String> {
    let mut ret_urls = Vec::new();
    for url in Prgrs::new(urls.iter(), urls.len()).set_length_move(Length::Proportional(0.5)) {
        if post_in_last_n_days(url, num_days) {
            ret_urls.push(url);
        }
    }
    ret_urls
}

fn get_commandline_arguments() -> (String, u32) {
    let matches = App::new("Reddit News Checker")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Checks if there are new posts on subreddits or users.")
        .arg(
            Arg::with_name("FILE")
                .help("Path to line separated url file.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("DAYS")
                .help("Specify in how many past days to search.")
                .required(true)
                .index(2),
        )
        .get_matches();
    let file_path = matches.value_of("FILE").unwrap();
    let days = value_t!(matches.value_of("DAYS"), u32).unwrap_or_else(|e| e.exit());
    (String::from(file_path), days)
}

fn main() {
    let (file_path, days) = get_commandline_arguments();
    let mut urls: Vec<String> = fs::read_to_string(file_path)
        .expect("Error reading file")
        .lines()
        .map(|s| String::from(s))
        .collect();
    urls = urls
        .into_iter()
        .filter(|url| url.starts_with("https://www.reddit.com/"))
        .collect();

    println!("Checking {} urls..", urls.len());
    let urls_with_news = get_urls_with_recent_posts(&urls, days);

    println!(
        "\nUrls with posts in the last {} days: ({}/{})",
        days,
        urls_with_news.len(),
        urls.len()
    );
    for url in urls_with_news {
        println!("{}", url);
    }
}
