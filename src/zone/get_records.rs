use reqwest::Client;
use tabled::builder::Builder;
use tabled::settings::Panel;

use crate::config;
use crate::errors::{TdnsError, TdnsErrorGenerator};
use crate::tables::TableStyles;
use crate::zone;
use crate::zone::records;

pub const CMD_NAME: &str = "Get Zone Records";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, clap::ValueEnum)]
pub enum ZoneRecordDetailLevel {
    Detailed,
    Summary,
}

pub struct GetRecordsCmd {
    config: config::Config,
    zone: String,
    domain: Option<String>,
    detail: ZoneRecordDetailLevel,
    table_style: TableStyles,
}

impl GetRecordsCmd {
    pub fn create(
        config_file: &str,
        zone_name: String,
        domain_name: Option<String>,
        detail: ZoneRecordDetailLevel,
        table_style: TableStyles,
    ) -> Result<GetRecordsCmd, config::ConfigFileError> {
        let cfg = config::read_config_file(config_file)?;
        Ok(GetRecordsCmd {
            config: cfg,
            zone: zone_name,
            domain: domain_name,
            detail: detail,
            table_style: table_style,
        })
    }

    pub async fn execute(&self) -> Result<(), TdnsError> {
        let client = match Client::builder().danger_accept_invalid_certs(true).build() {
            Ok(c) => c,
            Err(error) => {
                return self.make_http_err(error);
            }
        };

        let host = self.config.get_host();
        let mut url = format!(
            "{host}/api/zones/records/get?token={}&domain={}",
            self.config.get_token(),
            zone::helpers::get_target_domain(&self.zone, &self.domain)
        );
        if self.domain.is_none() {
            url = format!("{}&listZone=true", url);
        }

        let http_resp = match client.get(url).send().await {
            Ok(resp) => resp,
            Err(error) => {
                return self.make_http_err(error);
            }
        };

        let body = match http_resp.text().await {
            Ok(body) => body,
            Err(error) => {
                return self.make_http_err(error);
            }
        };

        let resp: records::ListZoneRecordsResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(error) => {
                return self.make_json_err(error);
            }
        };

        // let table_style = Style::ascii_rounded()
        //     .horizontals([(1, HorizontalLine::inherit(Style::ascii()).horizontal('-'))]);

        let mut zone_table = resp.records.zone.to_table();
        self.table_style.print_table(&mut zone_table);

        if self.detail == ZoneRecordDetailLevel::Summary {
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
            // table.with(table_style.clone());
            self.table_style.print_table(&mut table);
        } else {
            for record in resp.records.records {
                let mut table = record.to_detailed_table();
                table.with(Panel::header(record.name.clone()));
                // table.with(table_style.clone());
                self.table_style.print_table(&mut table);
            }
        }

        Ok(())
    }
}

impl TdnsErrorGenerator for GetRecordsCmd {
    fn get_command_name(&self) -> &str {
        CMD_NAME
    }
    fn get_host(&self) -> &str {
        self.config.get_host()
    }
}
