use std::path::PathBuf;
use tokio::fs::File;
use clap::ArgMatches;


pub async fn open_dictionary_file(matches: &ArgMatches) -> File {
    let file = matches.value_of("file").expect("missing attack dictionary file");
    let file_path = PathBuf::from(file).canonicalize().unwrap();
    if !file_path.is_file() {
        println!("[!] This is a not valid file.");
        use std::process;
        process::exit(1);
    }
    println!("[+] attack dictionary file {:?}", file_path);
    let file = File::open(file_path).await.expect("can`t not open file");
    file
}