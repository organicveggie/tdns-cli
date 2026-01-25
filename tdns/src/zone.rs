use reqwest::Client;

use crate::config;
use crate::errors::TdnsRequestError;
use tabled::{
    builder::Builder,
    settings::{Panel, Style, themes::BorderCorrection},
};

mod records;

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
                return Err(self.make_http_error(error));
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
                return Err(self.make_http_error(error));
            }
        };

        let body = match http_resp.text().await {
            Ok(body) => body,
            Err(error) => {
                return Err(self.make_http_error(error));
            }
        };

        let resp: records::ListZoneRecordsResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(error) => {
                return Err(TdnsRequestError::JsonError {
                    command: CMD_NAME.to_string(),
                    source: error,
                });
            }
        };

        println!("{}", resp.records.zone);

        // for record in resp.records.records {
        //     // println!("{}", record.name);

        //     let mut b = Builder::with_capacity(2, 2);
        //     // b.push_record(["Type", "Value"]);
        //     // b.push_record([record.name]);
        //     b.push_record([record.data.to_string(), record.data.value_summary()]);

        //     let mut table = b.build();
        //     // table.modify((0, 0), Span::column(2));
        //     table.with(Panel::header(record.name));
        //     table.with(BorderCorrection::span());

        //     // table.with(Style::modern());
        //     println!("{table}");
        //     // println!("{}", record);
        // }
        let mut b = Builder::with_capacity(resp.records.records.len(), 3);
        b.push_record(["Record", "Type", "Value"]);
        for record in resp.records.records {
            b.push_record([
                record.name,
                record.data.to_string(),
                record.data.value_summary(),
            ]);
        }

        let mut table = b.build();
        table.with(Style::modern());
        println!("{table}");

        Ok(())
    }

    fn make_http_error(&self, error: reqwest::Error) -> TdnsRequestError {
        TdnsRequestError::HttpRequestError {
            command: CMD_NAME.to_string(),
            host: self.config.get_host().to_string(),
            source: error,
        }
    }
}
