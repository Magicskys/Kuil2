use url::Url;
use tokio::task::{JoinHandle};
use std::fs::{File};
use std::path::PathBuf;
use std::io::{BufRead, BufReader};
use std::error::Error;
use std::time::{Duration};
use clap::ArgMatches;
use std::sync::atomic::Ordering::Relaxed;

use crate::attack::SubDirectory;

pub async fn send_payload(item: SubDirectory, matches: &ArgMatches<'_>) -> Result<(), Box<dyn Error>> {
    let file = matches.value_of("file").expect("missing attack dictionary file");
    let file_path = PathBuf::from(file).canonicalize().unwrap();
    if !file_path.is_file() {
        println!("[!] This is not a valid file.");
        use std::process;
        process::exit(1);
    }
    println!("[+] attack dictionary file {:?}", file_path);
    let parse = Url::parse(item.url.as_str()).expect("parse target error");
    let file = File::open(file_path).expect("can`t not open file");
    let reader = BufReader::new(file);

    let mut sub_tasks: Vec<JoinHandle<()>> = vec![];
    println!("[+] now scan....");
    for (_, line) in reader.lines().enumerate() {
        if let Ok(line) = line {
            if line.trim() != "" {
                if let Ok(line) = parse.join(&line) {
                    let attack = item.clone();
                    let handle = tokio::spawn(async move {
                        attack.attack(line.as_str()).await;
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
    println!("[+] subdirectory scan success {} error {}", item.success_num.load(Relaxed), item.error_num.load(Relaxed));
    Ok(())
}