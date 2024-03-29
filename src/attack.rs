use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::time::Duration;
use rand::Rng;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::task::{JoinHandle};
use trust_dns_resolver::config::{NameServerConfigGroup, ResolverConfig, ResolverOpts};
use trust_dns_resolver::{TokioAsyncResolver};
use url::Url;

#[derive(Debug, Clone)]
pub struct SubDirectory {
    pub client: Arc<Client>,
    pub url: Url,
    pub success_num: Arc<AtomicI32>,
    pub error_num: Arc<AtomicI32>,
}

#[derive(Debug, Clone)]
pub struct SubDomain {
    domain: String,
    success_num: Arc<AtomicI32>,
    resolver: Arc<TokioAsyncResolver>,
}

impl SubDomain {
    pub fn new(domain: String, dns: &str) -> Self {
        let domain = if domain.starts_with("http://") || domain.starts_with("https://") {
            domain.splitn(2, "://").nth(1).unwrap().to_string()
        } else {
            domain
        };

        let dns_ip = Ipv4Addr::from_str(&dns);
        let resolver_config = match dns_ip {
            Ok(dns_ip) => {
                let dns_addr = IpAddr::V4(dns_ip);
                let name_group = NameServerConfigGroup::from_ips_clear(&vec![dns_addr], 53, true);
                ResolverConfig::from_parts(None, vec![], name_group)
            }
            Err(_) => {
                println!("[!] DNS server IP address resolution error");
                ResolverConfig::default()
            }
        };
        let resolver_opts = ResolverOpts::default();
        let resolver = Arc::new(TokioAsyncResolver::tokio(resolver_config, resolver_opts).unwrap());

        println!("[+] Domain => {} DNS => {}", domain.clone(), dns);
        SubDomain {
            domain,
            success_num: Arc::new(AtomicI32::new(0)),
            resolver,
        }
    }

    async fn lookup(&self, target: String) -> bool {
        if let Ok(response) = self.resolver.as_ref().lookup_ip(target.clone()).await {
            let ips: Vec<IpAddr> = response.iter().collect();
            println!("\t{} {:?}", target, ips);
            self.success_num.as_ref().fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            true
        } else {
            false
        }
    }


    pub async fn verify_cdn(&self) -> bool {
        println!("[*] Verify whether the domain name is a CDN");
        let mut rng = rand::thread_rng();
        let charset = "abcdefghijklmnopqrstuvwxyz";
        let mut pass_count = 0;
        for _ in 0..5 {
            let random_string: String = (0..10)
                .map(|_| {
                    let index = rng.gen_range(0..charset.len());
                    charset.chars().nth(index).unwrap()
                })
                .collect();
            let target = random_string + "." + &self.domain.clone();
            if self.lookup(target).await {
                pass_count += 1;
            };
        }
        if pass_count == 5 {
            println!("[!] Suspected CND analysis, skipped");
            return true;
        }
        false
    }

    pub async fn detection(&self, file: File) {
        if !self.verify_cdn().await {
            println!("[+] Start brute force");
            let reader = BufReader::new(file);
            let mut lines = reader.lines();
            let mut sub_tasks: Vec<JoinHandle<()>> = vec![];
            while let Ok(Some(line)) = lines.next_line().await {
                let line_trim = line.trim();
                if line_trim != "" {
                    let target = line_trim.to_owned() + "." + &self.domain;
                    let self_clone = self.clone();
                    let future = async move {
                        self_clone.lookup(target).await;
                    };
                    let task = tokio::spawn(future);
                    sub_tasks.push(task)
                }
            }
            for i in sub_tasks {
                i.await.unwrap();
            }
        };
        println!("[+] subdomain scan success {}", self.success_num.load(std::sync::atomic::Ordering::Relaxed));
    }
}


impl SubDirectory {
    pub fn new(url: String, timeout: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .trust_dns(true)
            .build().expect("build request error");
        let url = Url::parse(url.as_str()).expect("parse url error");
        SubDirectory {
            client: Arc::new(client),
            url,
            success_num: Arc::new(AtomicI32::new(0)),
            error_num: Arc::new(AtomicI32::new(0)),
        }
    }

    pub async fn detection(&self, file: File) {
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut sub_tasks: Vec<JoinHandle<()>> = vec![];
        while let Ok(Some(line)) = lines.next_line().await {
            let line_trim = line.trim();
            if line_trim != "" {
                if let Ok(target) = self.url.join(line_trim) {
                    let clone_client = self.client.clone();
                    let success_clone = self.success_num.clone();
                    let handle = tokio::spawn(async move {
                        let response = clone_client.get(target.clone()).send().await;
                        match response {
                            Ok(response) => {
                                if response.status() != 404 {
                                    println!("[*] {} [{}]", target, response.status().as_str());
                                    success_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                            Err(_) => {}
                        }
                    });
                    sub_tasks.push(handle);
                }
            }
        }
        for i in sub_tasks {
            i.await.unwrap();
        }
        println!("[+] subdirectory scan success {}", self.success_num.load(std::sync::atomic::Ordering::Relaxed));
    }
}
