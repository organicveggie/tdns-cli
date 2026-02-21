use async_trait::async_trait;
use mockall::automock;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[automock]
#[async_trait]
pub trait TdnsClient {
    async fn get_body(
        &self,
        url: &str,
        query_params: &Option<QueryBuilder>,
    ) -> Result<String, reqwest::Error>;
    async fn post_body(
        &self,
        url: &str,
        body: &Option<String>,
        query_params: &Option<QueryBuilder>,
    ) -> Result<String, reqwest::Error>;
}

pub struct TdnsHttpClient {
    client: reqwest::Client,
}

impl TdnsHttpClient {
    pub fn new(ignore_invalid_certs: bool) -> Result<TdnsHttpClient, reqwest::Error> {
        let mut builder = reqwest::Client::builder();
        if ignore_invalid_certs {
            builder = builder.tls_danger_accept_invalid_certs(true);
        }
        let client = builder.build()?;
        Ok(TdnsHttpClient { client })
    }
}

#[async_trait]
impl TdnsClient for TdnsHttpClient {
    async fn get_body(
        &self,
        url: &str,
        query_params: &Option<QueryBuilder>,
    ) -> Result<String, reqwest::Error> {
        let mut request = self.client.get(url);
        if let Some(params) = query_params {
            request = request.query(&params);
        }
        request.send().await?.text().await
    }

    async fn post_body(
        &self,
        url: &str,
        body: &Option<String>,
        query_params: &Option<QueryBuilder>,
    ) -> Result<String, reqwest::Error> {
        let mut request = self.client.post(url);
        if let Some(body_str) = body {
            request = request.body(body_str.clone());
        }
        if let Some(params) = query_params {
            request = request.query(&params);
        }
        request.send().await?.text().await
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct QueryBuilder {
    params: BTreeMap<String, String>,
}

impl<K: Ord + std::fmt::Display, const N: usize> From<[(K, K); N]> for QueryBuilder {
    fn from(arr: [(K, K); N]) -> QueryBuilder {
        let mut params = BTreeMap::new();
        for (key, value) in arr.iter() {
            params.insert(key.to_string(), value.to_string());
        }
        QueryBuilder { params }
    }
}

impl<K: Ord + std::fmt::Display> From<Vec<(K, K)>> for QueryBuilder {
    fn from(vec: Vec<(K, K)>) -> QueryBuilder {
        let mut params = BTreeMap::new();
        for (key, value) in vec.iter() {
            params.insert(key.to_string(), value.to_string());
        }
        QueryBuilder { params }
    }
}

impl QueryBuilder {
    pub fn from_map(map: BTreeMap<String, String>) -> QueryBuilder {
        QueryBuilder { params: map }
    }

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

    pub fn clone_to_map(&self) -> BTreeMap<String, String> {
        self.params.clone()
    }
}

#[cfg(test)]
mod query_builder_tests {
    use super::*;

    #[test]
    fn test_one_param_to_map() {
        let qb = QueryBuilder::new().add_param("key1", "value1");
        assert_eq!(qb.clone_to_map(), BTreeMap::from([("key1".to_string(), "value1".to_string())]));
    }

    #[test]
    fn test_multiple_params() {
        let qb = QueryBuilder::new().add_param("key2", "value2").add_param("key1", "value1");
        assert_eq!(
            qb.clone_to_map(),
            BTreeMap::from([
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string())
            ])
        );
    }

    #[test]
    fn test_merge() {
        let qb1 = QueryBuilder::new().add_param("key1", "value1");
        let qb2 = QueryBuilder::new().add_param("key2", "value2");
        let merged = qb1.merge(qb2);
        assert_eq!(
            merged.clone_to_map(),
            BTreeMap::from([
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string())
            ])
        );
    }
}
