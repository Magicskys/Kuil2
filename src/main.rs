extern crate clap;

use std::error::Error;
use url::Url;
use std::sync::Arc;
use std::fs::{File};
use std::path::PathBuf;
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use clap::{Arg, App};
use reqwest;
use reqwest::{Client, StatusCode};
use std::time::{Duration, SystemTime};
use tokio::task::{JoinHandle};
use std::sync::atomic::{AtomicI32};
use std::sync::atomic::Ordering::Relaxed;


async fn get_url(client: &Client, url: &str) -> Result<StatusCode, reqwest::Error> {
    let response = client.get(url).send().await?;
    Ok(response.status())
}


#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Kuil2")
        .version("1.0")
        .author("https://github.com/Magicskys")
        .about("This is Kuil2 ,Kuil2 is a comprehensive hacking tool.")
        .arg(Arg::with_name("url")
            .short("u")
            .long("url")
            .help("target url\n\thttp://example.com")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .help("text file")
            .takes_value(true)
            .required(true))
        .get_matches();
    let file = matches.value_of("file").expect("missing attack dictionary file");
    let file_path = PathBuf::from(file).canonicalize().unwrap();
    if !file_path.is_file() {
        println!("[!] This is not a valid file.");
        use std::process;
        process::exit(1);
    }
    let target = matches.value_of("url").expect("missing attack target");
    println!("[+] attack target {}", target);
    println!("[+] attack dictionary file {:?}", file_path);

    let parse = Url::parse(target).expect("parse target error");
    let file = File::open(file_path).expect("can`t not open file");
    let reader = BufReader::new(file);
    let now_time = SystemTime::now();
    let mut sub_tasks: Vec<JoinHandle<()>> = vec![];
    let error_num = Arc::new(AtomicI32::new(0));
    let success_num = Arc::new(AtomicI32::new(0));
    let client = Arc::new(reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .trust_dns(true)
        .build()?);

    for (index, line) in reader.lines().enumerate() {
        if let Ok(line) = line {
            if line.trim() != "" {
                if let Ok(line) = parse.join(&line) {
                    let success_n = success_num.clone();
                    let error_n = error_num.clone();
                    let client_clone = client.clone();
                    let handle = tokio::spawn(async move {
                        let status_code = get_url(client_clone.as_ref(), line.as_str()).await;
                        match status_code {
                            Ok(status_code) => {
                                println!("[{}] {} [{}]", index, line.as_str(), status_code.as_u16());
                                io::stdout().lock().flush().unwrap();
                                success_n.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                            Err(e) => {
                                println!("[{}] {} [{}]", index, line.as_str(), e);
                                io::stdout().lock().flush().unwrap();
                                error_n.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    });
                    sub_tasks.push(handle)
                }
            }
        }
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
    for i in sub_tasks {
        i.await.unwrap();
    }
    println!("[+] total time {} seconds", SystemTime::now().duration_since(now_time).unwrap_or(Duration::from_secs(0)).as_secs());
    println!("[+] success {} error {}", success_num.load(Relaxed), error_num.load(Relaxed));
    Ok(())
}
