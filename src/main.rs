extern crate clap;

mod common;
mod attack;

use clap::{Arg, App, Command};
use std::error::Error;
use std::process;
use std::time::{Duration, SystemTime};
use crate::attack::{SubDirectory, SubDomain};
use crate::common::{open_dictionary_file};


#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Kuil2")
        .version("1.0")
        .author("https://github.com/Magicskys")
        .about("This is Kuil2 ,Kuil2 is a comprehensive hacking tool.")
        .arg(Arg::with_name("url")
            .short('u')
            .long("url")
            .help("target url\n\thttp://example.com")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("timeout")
            .long("timeout")
            .default_value("30")
            .help("overtime time")
            .takes_value(true)
            .required(false))
        .subcommands(vec![
            Command::new("directory").about("Subdirectory blasting")
                .arg(Arg::new("file")
                    .short('f')
                    .long("file")
                    .help("text file")
                    .takes_value(true)
                    .required(true)),
            Command::new("domain").about("Subdomain blasting")
                .arg(Arg::new("file")
                    .short('f')
                    .long("file")
                    .help("dictionary file")
                    .takes_value(true)
                    .required(true))
                .arg(Arg::new("dns")
                    .long("dns")
                    .help("DNS server for resolve")
                    .default_value("8.8.8.8")
                    .default_missing_value("8.8.8.8")
                    .takes_value(true)),
        ])
        .get_matches();
    let now_time = SystemTime::now();
    let target = matches.value_of("url").expect("missing attack target");
    let timeout = matches.value_of("timeout").expect("timeout needs to be a number").parse::<u64>()?;
    println!("[+] PID {}", process::id());
    println!("[+] detection target {}", target);
    match matches.subcommand() {
        Some(("directory", sub_m)) => {
            println!("[+] start subdirectory detection");
            let module = SubDirectory::new(target.into(), timeout);
            let file = open_dictionary_file(sub_m).await;
            module.detection(file).await;
        }
        Some(("domain", sub_m)) => {
            println!("[+] start subdomain detection");
            let module = SubDomain::new(target.into());
            let file = open_dictionary_file(sub_m).await;
            let dns = sub_m.value_of("dns").expect("missing dns server ip address");
            module.detection(file, dns).await;
        }
        _ => {
            println!("[*] start port scan");
            use std::process;
            process::exit(1);
        }
    }
    println!("[+] total time {} seconds", SystemTime::now().duration_since(now_time).unwrap_or(Duration::from_secs(0)).as_secs());
    Ok(())
}
