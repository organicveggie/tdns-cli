use reqwest::Client;
use serde::Deserialize;
use std::fmt;

use crate::config;
use crate::errors::TdnsRequestError;

const CMD_NAME: &str = "Get Zone Records";

#[derive(Debug, thiserror::Error)]
pub enum ZoneError {
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

#[derive(Debug, Deserialize)]
#[serde(rename = "response")]
pub struct ListZoneRecordsResponse {
    // pub zone: zones::Zone,
    #[serde(rename = "response")]
    pub records: RecordsList,
}

#[derive(Debug, Deserialize)]
pub struct RecordsList {
    pub records: Vec<ZoneRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "rData")]
pub enum RecordData {
    #[serde(rename_all = "camelCase")]
    A {
        ip_address: String,
    },
    #[serde(rename_all = "camelCase")]
    AAAA {
        ip_address: String,
    },
    CNAME {
        cname: String,
    },
    #[serde(rename_all = "camelCase")]
    NS {
        name_server: String,
    },
    #[serde(rename_all = "camelCase")]
    SOA {
        primary_name_server: String,
        serial: u64,
    },
    #[serde(untagged)]
    Unknown {
        #[serde(rename = "type")]
        record_type: String,
        #[serde(rename = "rData")]
        r_data: serde_json::Value,
    },
}

impl fmt::Display for RecordData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordData::A { ip_address } => write!(f, "A {}", ip_address),
            RecordData::AAAA { ip_address } => write!(f, "AAAA {}", ip_address),
            RecordData::CNAME { cname } => write!(f, "CNAME {}", cname),
            RecordData::NS { name_server } => write!(f, "NS {}", name_server),
            RecordData::SOA {
                primary_name_server,
                serial,
            } => write!(f, "SOA {} / serial {}", primary_name_server, serial),
            _ => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneRecord {
    pub name: String,

    pub ttl: u32,
    pub disabled: bool,
    pub dnssec_status: String,

    #[serde(flatten)]
    pub data: RecordData,

    pub ttl_string: Option<String>,
    pub last_used_on: Option<String>,
    pub last_modified: Option<String>,
    pub expiry_ttl: Option<u32>,
    pub expiry_ttl_string: Option<String>,
}

impl fmt::Display for ZoneRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        writeln!(f, "  {}", self.data)?;
        Ok(())
    }
}

fn get_target_domain(zone: &str, domain: &Option<String>) -> String {
    if let Some(domain_name) = domain {
        if domain_name.ends_with(zone) {
            return domain_name.to_string();
        }
        return format!("{}.{}", domain_name, zone);
    }

    zone.to_string()
}

pub struct GetRecordsCmd {
    config: config::Config,
    zone: String,
    domain: Option<String>,
}

impl GetRecordsCmd {
    pub fn create(
        config_file: &str,
        zone_name: String,
        domain_name: Option<String>,
    ) -> Result<GetRecordsCmd, config::ConfigFileError> {
        let cfg = config::read_config_file(config_file)?;
        Ok(GetRecordsCmd {
            config: cfg,
            zone: zone_name,
            domain: domain_name,
        })
    }

    pub async fn execute(&self) -> Result<(), TdnsRequestError> {
        let client = match Client::builder().danger_accept_invalid_certs(true).build() {
            Ok(c) => c,
            Err(error) => {
                return Err(TdnsRequestError::HttpRequestError {
                    command: CMD_NAME.to_string(),
                    host: self.config.get_host().to_string(),
                    source: error,
                });
            }
        };

        let host = self.config.get_host();
        let mut url = format!(
            "{host}/api/zones/records/get?token={}&domain={}",
            self.config.get_token(),
            get_target_domain(&self.zone, &self.domain)
        );
        if self.domain.is_none() {
            url = format!("{}&listZone=true", url);
        }

        let http_resp = match client.get(url).send().await {
            Ok(resp) => resp,
            Err(error) => {
                return Err(TdnsRequestError::HttpRequestError {
                    command: CMD_NAME.to_string(),
                    host: self.config.get_host().to_string(),
                    source: error,
                });
            }
        };

        let body = match http_resp.text().await {
            Ok(body) => body,
            Err(error) => {
                return Err(TdnsRequestError::HttpRequestError {
                    command: CMD_NAME.to_string(),
                    host: self.config.get_host().to_string(),
                    source: error,
                });
            }
        };

        let resp: ListZoneRecordsResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(error) => {
                return Err(TdnsRequestError::JsonError {
                    command: CMD_NAME.to_string(),
                    source: error,
                });
            }
        };

        for record in resp.records.records {
            println!("{}", record);
        }

        Ok(())
    }
}
