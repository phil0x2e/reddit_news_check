extern crate argparse;
extern crate reqwest;
extern crate prgrs;

use prgrs::{Prgrs, Length, writeln};
use argparse::{ArgumentParser, Store};
use reqwest::header::COOKIE;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

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
        Ok(days) => {
            days.contains(&true)
        }
    }
}

fn get_urls_with_recent_posts(urls: &Vec<String>, num_days: u32) -> Vec<String> {
    let mut ret_urls = Vec::new();
    for  url in Prgrs::new(urls.iter(), urls.len()).set_length_move(Length::Proportional(0.5)) {
        if post_in_last_n_days(url, num_days) {
            ret_urls.push(url.clone());
        }
    }
    ret_urls
}

fn main() {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    let mut file_path = String::new();
    let mut days: u32 = 0;
    let description_str = format!("Reddit news checker Version {}", VERSION);
    {
        let mut ap = ArgumentParser::new();
        ap.set_description(&description_str);
        ap.refer(&mut file_path).required().add_argument(
            "FILE",
            Store,
            "Path to Line separated url file.",
        );
        ap.refer(&mut days).required().add_argument(
            "DAYS",
            Store,
            "Specify in how many past days to search.",
        );
        ap.parse_args_or_exit();
    }
    let file = File::open(file_path);
    let file = match file {
        Ok(file) => file,
        Err(_error) => {
            println!("File not found, or could not be opened.");
            return;
        }
    };
    let buf = BufReader::new(file);
    let mut urls = buf
        .lines()
        .map(|x| x.expect("Error reading line from file"))
        .collect::<Vec<String>>();
    urls = urls
        .into_iter()
        .filter(|url| url.starts_with("https://www.reddit.com/"))
        .collect::<Vec<String>>();
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
