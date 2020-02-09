extern crate reqwest;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};
use std::env;

fn check_url(url : &str, max_days: u32) -> Vec<bool>{
    let res = reqwest::Client::new()
        .get(url).send().unwrap().text().unwrap();
    
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
    let args: Vec<String> = env::args().collect();
    if args.len() != 2{
        println!("Please supply path to file with urls.");
    }else{
        let file = File::open(&args[1]);
        let file = match file{
            Ok(file) => file,
            Err(_error) => {
                println!("File not found, or could not be opened.");
                return;
            }
        };
        let buf = BufReader::new(file);
        let urls = buf.lines().map(|x| x.unwrap()).collect::<Vec<String>>();

        let n = 7;

        let urls_with_news = get_urls_with_recent_posts(&urls, n);
        
        println!("====================\nUrls mit posts in den letzten {} Tagen: ", n);
        for url in urls_with_news{
            println!("{}", url);
        }
    }

}
