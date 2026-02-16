use tabled::builder::Builder;
use tabled::settings::Panel;

use crate::client::QueryBuilder;
use crate::errors::TdnsError;
use crate::tables::TableStyles;
use crate::zone;
use crate::zone::records;
use crate::{config, errors};

pub const CMD_NAME: &str = "Get Zone Records";
pub const API_GET_RECORDS_PATH: &str = "/api/zones/records/get";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, clap::ValueEnum)]
pub enum ZoneRecordDetailLevel {
    Detailed,
    Summary,
}

pub struct GetRecordsCmd {
    zone: String,
    domain: Option<String>,
    detail: ZoneRecordDetailLevel,
    table_style: TableStyles,
}

impl GetRecordsCmd {
    pub fn create(
        zone_name: String,
        domain_name: Option<String>,
        detail: ZoneRecordDetailLevel,
        table_style: TableStyles,
    ) -> GetRecordsCmd {
        GetRecordsCmd {
            zone: zone_name,
            domain: domain_name,
            detail: detail,
            table_style: table_style,
        }
    }

    pub async fn execute(
        &self,
        config_file: &str,
        app_config: &config::ApplicationConfig,
    ) -> Result<(), TdnsError> {
        let cfg = match app_config.config_manager.read_config_file(config_file) {
            Ok(c) => c,
            Err(error) => {
                return Err(errors::make_config_error(CMD_NAME, error));
            }
        };

        let mut query_params = QueryBuilder::new()
            .add_param("token", cfg.get_token())
            .add_param("domain", &zone::helpers::get_target_domain(&self.zone, &self.domain));
        if self.domain.is_none() {
            query_params = query_params.add_param("listZone", "true");
        }

        let host = cfg.get_host();
        let url = format!("{host}{API_GET_RECORDS_PATH}");

        let body = match app_config.tdns_client.get_body(&url, &Some(query_params)).await {
            Ok(body) => body,
            Err(error) => {
                return Err(errors::make_http_error(CMD_NAME, host, error));
            }
        };

        let resp: records::ListZoneRecordsResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(error) => {
                return Err(errors::make_json_error(CMD_NAME, error));
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
                b.push_record([record.name, record.data.to_string(), record.data.value_summary()]);
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
