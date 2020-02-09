extern crate reqwest;
extern crate argparse;

use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use argparse::{ArgumentParser, Store};

fn check_url(url : &str, max_days: u32) -> Vec<bool>{
    println!("{}", url);
    let res = reqwest::get(url).expect("Couldn't get url")
    .text()
    .expect("Couldn't extract text from url");

    let mut days = Vec::new();
    for d in 0..max_days+1{
        if d == 0{
            days.push(res.contains(" hours ago") || res.contains(" hour ago"));
        }
        else if d == 1{
            days.push(res.contains(">1 day ago"));
        }else{
            days.push(res.contains(format!(">{} days ago", d).as_str()));
        }
    }
    days
}


fn post_in_last_n_days(url: &str, n: u32) -> bool{
    let days = check_url(url, n);
    days.contains(&true)
}

fn get_urls_with_recent_posts(urls: &Vec<String>, num_days: u32) -> Vec<String>{
    let mut ret_urls = Vec::new();
    for (i, url) in urls.iter().enumerate(){
        println!("Url {} von {} wird geprüft", i+1 ,urls.len());
        if post_in_last_n_days(url, num_days){
            ret_urls.push(url.clone());
        }
    }
    ret_urls
}

fn main() {
    let mut file_path = "World".to_string();
    let mut days : u32 = 0;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Reddit Profile news checker");
        ap.refer(&mut file_path)
                .required().add_argument("FILE", Store,
                "Path to Line separated url file.");
        ap.refer(&mut days)
                .required().add_argument("DAYS", Store,
                "Specify in how many past days to search.");
        ap.parse_args_or_exit();
    }

        let file = File::open(file_path);
        let file = match file{
            Ok(file) => file,
            Err(_error) => {
                println!("File not found, or could not be opened.");
                return;
            }
        };
        let buf = BufReader::new(file);
        let urls = buf.lines().map(|x| x.unwrap()).collect::<Vec<String>>();


        let urls_with_news = get_urls_with_recent_posts(&urls, days);

        println!("====================\nUrls mit posts in den letzten {} Tagen: ", days);
        for url in urls_with_news{
            println!("{}", url);
        }
}
