use async_trait::async_trait;
use mockall::automock;
use std::collections::BTreeMap;

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
        let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build()?;
        Ok(TdnsHttpClient { client })
    }
}

#[async_trait]
impl TdnsClient for TdnsHttpClient {
    async fn get_body(&self, url: &str) -> Result<String, reqwest::Error> {
        let http_resp = self.client.get(url).send().await?;
        http_resp.text().await
    }

    async fn post_body(&self, url: &str, body: &str) -> Result<String, reqwest::Error> {
        let http_resp = self.client.post(url).body(body.to_string()).send().await?;
        http_resp.text().await
    }
}

#[derive(Clone, Debug)]
pub struct QueryBuilder {
    params: BTreeMap<String, String>,
}

impl QueryBuilder {
    pub fn new() -> QueryBuilder {
        QueryBuilder { params: BTreeMap::new() }
    }

    pub fn add_param(mut self, key: &str, value: &str) -> QueryBuilder {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn merge(mut self, other: QueryBuilder) -> QueryBuilder {
        for (key, value) in other.params {
            self.params.insert(key, value);
        }
        self
    }

    pub fn build(self) -> String {
        let mut query = String::new();
        for (key, value) in self.params {
            if !query.is_empty() {
                query.push('&');
            }
            query.push_str(&format!("{}={}", key, value));
        }
        query
    }
}

#[cfg(test)]
mod query_builder_tests {
    use super::*;

    #[test]
    fn test_one_param() {
        let query = QueryBuilder::new().add_param("key1", "value1").build();
        assert_eq!(query, "key1=value1");
    }

    #[test]
    fn test_multiple_params() {
        let query =
            QueryBuilder::new().add_param("key1", "value1").add_param("key2", "value2").build();
        assert_eq!(query, "key1=value1&key2=value2");
    }
}
