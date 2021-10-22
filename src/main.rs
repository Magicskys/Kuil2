extern crate clap;

mod common;
mod attack;

use clap::{Arg, App, SubCommand};
use std::error::Error;
use std::time::{Duration, SystemTime};
use crate::attack::{SubDirectory};
use crate::common::send_payload;


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
        .arg(Arg::with_name("timeout")
            .long("timeout")
            .default_value("30")
            .help("overtime time")
            .takes_value(true)
            .required(false))
        .subcommand(
            SubCommand::with_name("directory").about("scan sub domain")
                .arg(Arg::with_name("file")
                    .short("f")
                    .long("file")
                    .help("text file")
                    .takes_value(true)
                    .required(true))
        )
        .get_matches();
    let now_time = SystemTime::now();
    let target = matches.value_of("url").expect("missing attack target");
    let timeout = matches.value_of("timeout").expect("timeout needs to be a number").parse::<u64>()?;
    println!("[+] attack target {}", target);
    match matches.subcommand() {
        ("directory", Some(sub_m)) => {
            println!("[+] start subdirectory scan");
            let item = SubDirectory::new(target.into(), timeout)?;
            send_payload(item, sub_m).await?
        }
        ("domain", Some(sub_m)) => {
            println!("[+] start sub domain scan");
            todo!()
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
