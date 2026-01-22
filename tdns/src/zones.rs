use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config;

const LIST_CMD: &str = "List Zones";

#[derive(Debug, thiserror::Error)]
pub enum ZonesError {
    #[error("{command} HTTP request to {host} failed")]
    HttpRequestError {
        command: String,
        host: String,
        source: reqwest::Error,
    },
    #[error("{command} JSON deserialization failed")]
    JsonError {
        command: String,
        source: serde_json::Error,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "response")]
pub struct ListZonesResponse {
    #[serde(rename = "response")]
    pub zone_list: ZoneList,
    pub server: String,
    pub status: String,

    #[serde(rename = "pageNumber")]
    pub page_number: Option<u32>,
    #[serde(rename = "totalPages")]
    pub total_pages: Option<u32>,
    #[serde(rename = "totalZones")]
    pub total_zones: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneList {
    pub zones: Vec<Zone>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub name: String,

    #[serde(rename = "type")]
    pub zone_type: String, // Primary or Secondary

    pub internal: bool,
    pub dnssec_status: String,
    pub soa_serial: u32,
    pub last_modified: String, // "2022-02-26T07:57:08.1842183Z"
    pub disabled: bool,

    pub catalog: Option<String>,
    pub expiry: Option<String>, // "2022-02-26T07:57:08.1842183Z"
    pub is_expired: Option<bool>,
    pub notify_failed: Option<bool>,
    pub notify_failed_for: Option<Vec<String>>,
    pub sync_failed: Option<bool>,
}

pub struct ListCmd {
    config: config::Config,
}

impl ListCmd {
    pub fn create(config_file: &str) -> Result<ListCmd, config::ConfigFileError> {
        let cfg = config::read_config_file(config_file)?;
        Ok(ListCmd { config: cfg })
    }

    pub async fn execute(&self) -> Result<(), ZonesError> {
        let client = match Client::builder().danger_accept_invalid_certs(true).build() {
            Ok(c) => c,
            Err(error) => {
                return Err(ZonesError::HttpRequestError {
                    command: LIST_CMD.to_string(),
                    host: self.config.get_host().to_string(),
                    source: error,
                });
            }
        };

        let host = self.config.get_host();
        let base_url = format!("{host}/api/zones/list?token={}", self.config.get_token());

        let http_resp = match client.get(base_url).send().await {
            Ok(resp) => resp,
            Err(error) => {
                return Err(ZonesError::HttpRequestError {
                    command: LIST_CMD.to_string(),
                    host: self.config.get_host().to_string(),
                    source: error,
                });
            }
        };

        let body = match http_resp.text().await {
            Ok(body) => body,
            Err(error) => {
                return Err(ZonesError::HttpRequestError {
                    command: LIST_CMD.to_string(),
                    host: self.config.get_host().to_string(),
                    source: error,
                });
            }
        };

        let resp: ListZonesResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(error) => {
                return Err(ZonesError::JsonError {
                    command: LIST_CMD.to_string(),
                    source: error,
                });
            }
        };

        for zone in resp.zone_list.zones {
            println!("Zone: {}, Type: {}", zone.name, zone.zone_type);
        }

        Ok(())
    }
}
