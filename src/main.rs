extern crate clap;
extern crate prgrs;
extern crate reqwest;

use clap::{crate_authors, crate_version, value_t, App, Arg};
use futures::executor::block_on;
use futures::future::join_all;
use prgrs::{writeln, Length, Prgrs};
use reqwest::header::COOKIE;
use std::fs;
use std::sync::Arc;

async fn check_url(
    client: Arc<reqwest::Client>,
    url: &str,
    max_days: u32,
    warn: bool,
) -> Result<Vec<bool>, reqwest::Error> {
    let res = client.get(url).header(COOKIE, "over18=1").send().await?;
    if res.status() == 404 {
        if warn {
            writeln(&format!("Warning: {} does not exist.", url))
                .expect("Something went wrong printing warning");
        }
        return Ok(Vec::new());
    }
    let text = res.text().await?;

    let mut days = Vec::new();
    for d in 0..max_days + 1 {
        if d == 0 {
            days.push(
                text.contains(" hours ago</a>")
                    || text.contains(" hour ago</a>")
                    || text.contains(" minutes ago</a>"),
            );
        } else if d == 1 {
            days.push(text.contains(">1 day ago</a>"));
        } else {
            days.push(text.contains(format!(">{} days ago</a>", d).as_str()));
        }
    }
    Ok(days)
}

async fn post_in_last_n_days<'a>(
    client: Arc<reqwest::Client>,
    url: String,
    n: u32,
    warn: bool,
) -> (String, bool) {
    match check_url(client, &url, n, warn).await {
        Err(e) => {
            writeln(&format!("Error: {}", e)).ok();
            (String::from(url), false)
        }
        Ok(days) => (String::from(url), days.contains(&true)),
    }
}

async fn get_urls_with_recent_posts(urls: Vec<String>, num_days: u32, warn: bool) -> Vec<String> {
    let client = Arc::new(reqwest::Client::new());
    let mut tasks = Vec::with_capacity(urls.len());
    for url in Prgrs::new(urls.iter(), urls.len()).set_length_move(Length::Proportional(0.5)) {
        tasks.push(tokio::spawn(post_in_last_n_days(
            client.clone(),
            String::from(url),
            num_days,
            warn,
        )));
    }
    let res: Vec<(String, bool)> = join_all(tasks)
        .await
        .into_iter()
        .map(Result::unwrap)
        .collect();
    let ret_urls: Vec<String> = res
        .into_iter()
        .filter_map(|r| if r.1 { Some(r.0) } else { None })
        .collect();

    ret_urls
}

struct Config {
    pub file_path: String,
    pub days: u32,
    pub warn: bool,
}

fn get_commandline_arguments() -> Config {
    let description = "This tool checks if there are new posts on subreddits or users.\n\nJust pass a file with line separated urls to subreddits, reddit users etc. and a time interval in which to search and it will return all urls, that have a new post in the specified time.\nFor it to work as expected you should specify the url of a subreddit with /new/ at the end and for a user with /posts/.";
    let matches = App::new("Reddit News Checker")
        .version(crate_version!())
        .author(crate_authors!())
        .about(description)
        .arg(
            Arg::with_name("FILE")
                .help("Path to line separated url file.\nUrls have to start with https://www.reddit.com/, otherwise they will be ignored.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("DAYS")
                .help("Specify in how many past days to search for new posts.\nA value of 0 means 24 hours or less. A value of 1 means 1 day or less etc.")
                .required(true)
                .index(2),
        ).arg(
            Arg::with_name("warn")
            .short("w")
            .required(false)
            .takes_value(false)
            .help("Warn about dead links.")
        )
        .get_matches();

    let file_path = matches.value_of("FILE").unwrap();
    let days = value_t!(matches.value_of("DAYS"), u32).unwrap_or_else(|e| e.exit());
    let warn = matches.is_present("warn");
    Config {
        file_path: String::from(file_path),
        days,
        warn,
    }
}

#[tokio::main]
async fn main() {
    let conf = get_commandline_arguments();
    let mut urls: Vec<String> = fs::read_to_string(conf.file_path)
        .expect("Error reading file")
        .lines()
        .map(|s| String::from(s))
        .collect();
    urls.retain(|url| url.starts_with("https://www.reddit.com/"));
    let total_len = urls.len();
    println!("Checking {} urls..", urls.len());
    let urls_with_news = block_on(get_urls_with_recent_posts(urls, conf.days, conf.warn));

    println!(
        "\nUrls with posts in the last {} days: ({}/{})",
        conf.days,
        urls_with_news.len(),
        total_len
    );
    for url in urls_with_news {
        println!("{}", url);
    }
}
