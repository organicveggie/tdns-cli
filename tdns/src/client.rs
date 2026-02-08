use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub trait TdnsClient {
    async fn get_body(&self, url: &str) -> Result<String, reqwest::Error>;
    async fn post_body(&self, url: &str, body: &str) -> Result<String, reqwest::Error>;
}

pub struct TdnsHttpClient {
    client: reqwest::Client,
}

impl TdnsHttpClient {
    pub fn new() -> Result<TdnsHttpClient, reqwest::Error> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        Ok(TdnsHttpClient { client })
    }
}

#[async_trait]
impl TdnsClient for TdnsHttpClient {
    async fn get_body(&self, url: &str) -> Result<String, reqwest::Error> {
        let http_resp = self.client.get(url).send().await?;
        println!("Received HTTP response: {:?}", http_resp);
        http_resp.text().await
    }

    async fn post_body(&self, url: &str, body: &str) -> Result<String, reqwest::Error> {
        let http_resp = self.client.post(url).body(body.to_string()).send().await?;
        http_resp.text().await
    }
}
