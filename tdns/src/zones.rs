use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::config;

const LIST_CMD: &str = "List Zones";

pub enum ZoneSortMode {
    Unsorted,
    AlphabeticalAscending,
    AlphabeticalDescending,
}

impl ZoneSortMode {
    pub fn from_option(sort: &Option<bool>) -> ZoneSortMode {
        match sort {
            Some(b) => {
                if *b {
                    ZoneSortMode::AlphabeticalAscending
                } else {
                    ZoneSortMode::AlphabeticalDescending
                }
            }
            _ => ZoneSortMode::Unsorted,
        }
    }
}

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

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Zone: {}", self.name)?;
        write!(f, "  Type: {}", self.zone_type)?;

        if self.disabled {
            write!(f, " (DISABLED)")?;
        }
        if self.internal {
            write!(f, " (INTERNAL)")?;
        }
        writeln!(f, "")?;

        writeln!(f, "  dnssecStatus: {}", self.dnssec_status)?;
        writeln!(f, "  soaSerial: {}", self.soa_serial)?;
        writeln!(f, "  lastModified: {}", self.last_modified)?;

        if let Some(ref catalog) = self.catalog {
            writeln!(f, "  catalog: {}", catalog)?;
        }

        if let Some(ref expiry) = self.expiry {
            writeln!(f, "  expiry: {}", expiry)?;
        }

        if self.is_expired.is_some_and(|b| b) {
            writeln!(f, "  EXPIRED")?;
        }
        if self.sync_failed.is_some_and(|b| b) {
            writeln!(f, "  SYNC FAILED")?;
        }

        if self.notify_failed.is_some_and(|b| b) {
            writeln!(f, "  NOTIFY FAILED")?;
        }
        if let Some(ref notify) = self.notify_failed_for {
            writeln!(f, "  failed for: {}", notify.join(", "))?;
        }

        Ok(())
    }
}

pub struct ListCmd {
    config: config::Config,
    sort: ZoneSortMode,
}

impl ListCmd {
    pub fn create(
        config_file: &str,
        sort: ZoneSortMode,
    ) -> Result<ListCmd, config::ConfigFileError> {
        let cfg = config::read_config_file(config_file)?;
        Ok(ListCmd {
            config: cfg,
            sort: sort,
        })
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

        let mut resp: ListZonesResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(error) => {
                return Err(ZonesError::JsonError {
                    command: LIST_CMD.to_string(),
                    source: error,
                });
            }
        };

        match self.sort {
            ZoneSortMode::AlphabeticalAscending => {
                resp.zone_list.zones.sort_by(|a, b| a.name.cmp(&b.name))
            }
            ZoneSortMode::AlphabeticalDescending => {
                resp.zone_list.zones.sort_by(|a, b| b.name.cmp(&a.name))
            }
            _ => (),
        }

        for zone in resp.zone_list.zones {
            println!("{}", zone);
        }

        Ok(())
    }
}
