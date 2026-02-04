use reqwest::Client;
use serde::Deserialize;

use crate::config;
use crate::errors::{TdnsError, TdnsErrorGenerator};
use crate::zone::enums::ZoneRecordType;

pub const CMD_NAME: &str = "Add Zone Record";

#[derive(Debug, Deserialize)]
#[serde(rename = "response")]
pub struct AddRecordResponse {
    pub status: String,
}

pub struct AddRecordCmd {
    config: config::Config,
    zone: String,
    domain: String,
    record_type: ZoneRecordType,
}

impl AddRecordCmd {
    pub fn create(
        config_file: &str,
        zone: String,
        domain: String,
        record_type: ZoneRecordType,
    ) -> Result<AddRecordCmd, config::ConfigFileError> {
        let cfg = config::read_config_file(config_file)?;
        Ok(AddRecordCmd {
            config: cfg,
            zone,
            domain,
            record_type,
        })
    }

    pub async fn execute(&self) -> Result<(), TdnsError> {
        let client = match Client::builder().danger_accept_invalid_certs(true).build() {
            Ok(c) => c,
            Err(error) => {
                return Err(self.make_http_error(error));
            }
        };

        let host = self.config.get_host();
        let url = format!(
            "{host}/api/zones/records/add?token={}&domain={}&zone={}&recordType={}",
            self.config.get_token(),
            self.domain,
            self.zone,
            self.record_type
        );

        let http_resp = match client.get(url).send().await {
            Ok(resp) => resp,
            Err(error) => {
                return Err(self.make_http_error(error));
            }
        };

        let body = match http_resp.text().await {
            Ok(body) => body,
            Err(error) => {
                return Err(self.make_http_error(error));
            }
        };

        let resp: AddRecordResponse = match serde_json::from_str(&body) {
            Ok(resp) => resp,
            Err(error) => {
                return Err(self.make_json_error(error));
            }
        };

        println!("Response status: {}", resp.status);
        Ok(())
    }
}

impl TdnsErrorGenerator for AddRecordCmd {
    fn get_command_name(&self) -> &str {
        CMD_NAME
    }

    fn get_host(&self) -> &str {
        self.config.get_host()
    }
}
