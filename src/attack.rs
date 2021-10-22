use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::time::Duration;
use reqwest::Client;


#[derive(Debug, Clone)]
pub struct SubDirectory {
    pub client: Arc<Client>,
    pub url: String,
    pub success_num: Arc<AtomicI32>,
    pub error_num: Arc<AtomicI32>,
}

#[derive(Debug, Clone)]
pub struct SubDomain {}

impl SubDirectory {
    pub fn new(url: String, timeout: u64) -> Result<Self, reqwest::Error> {
        Ok(SubDirectory {
            client: Arc::new(reqwest::Client::builder()
                .timeout(Duration::from_secs(timeout))
                .trust_dns(true)
                .build()?),
            url,
            success_num: Arc::new(AtomicI32::new(0)),
            error_num: Arc::new(AtomicI32::new(0)),
        })
    }
    pub async fn attack(&self, target: &str) {
        let response = self.client.get(target).send().await;
        match response {
            Ok(response) => {
                println!("[*] {} [{}]", target, response.status());
                self.success_num.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            Err(e) => {
                println!("[!] {} [{}]", target, e);
                self.error_num.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }
}
