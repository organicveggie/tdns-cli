use mockall::predicate::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use tabled::Table;
use tabled::builder::Builder;
use tabled::settings::Panel;

use crate::cli;
use crate::client::TdnsClient;
use crate::config;
use crate::errors::{TdnsError, TdnsErrorGenerator};
use crate::tables::TableStyles;

const CMD_NAME: &str = "List Zones";
pub const API_LIST_ZONES_PATH: &str = "/api/zones/list";

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

    pub fn from_sort_order(sort_order: &cli::SortOrder) -> ZoneSortMode {
        match sort_order {
            cli::SortOrder::Unsorted => ZoneSortMode::Unsorted,
            cli::SortOrder::Ascending => ZoneSortMode::AlphabeticalAscending,
            cli::SortOrder::Descending => ZoneSortMode::AlphabeticalDescending,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "response")]
pub struct ListZonesResponse {
    #[serde(rename = "response")]
    pub zone_list: Option<ZoneList>,
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

impl Zone {
    pub fn to_table(&self) -> Table {
        let mut b = Builder::with_capacity(5, 2);
        if self.disabled || self.internal {
            let status = if self.disabled {
                if self.internal {
                    "DISABLED (INTERNAL)"
                } else {
                    "DISABLED"
                }
            } else {
                "(INTERNAL)"
            };

            b.push_record(["Status", status]);
        }
        b.push_record(["dnssecStatus", self.dnssec_status.as_str()]);
        b.push_record(["soaSerial", format!("{}", self.soa_serial).as_str()]);
        b.push_record(["lastModified", self.last_modified.as_str()]);
        if let Some(ref catalog) = self.catalog {
            b.push_record(["catalog", catalog]);
        }
        if let Some(ref expiry) = self.expiry {
            b.push_record(["expiry", expiry]);
        }

        let mut table = b.build();
        table.with(Panel::header(self.name.clone()));
        table
    }
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

pub struct ListCmd<T: TdnsClient> {
    client: T,
    config: config::Config,
    output_format: cli::OutputFormat,
    sort: ZoneSortMode,
    table_style: TableStyles,
}

impl<T: TdnsClient> ListCmd<T> {
    pub fn create(
        config_manager: impl config::ConfigManager,
        client: T,
        config_file: &str,
        output_format: &cli::OutputFormat,
        sort: ZoneSortMode,
        table_style: TableStyles,
    ) -> Result<ListCmd<T>, config::ConfigFileError> {
        let cfg = config_manager.read_config_file(config_file)?;
        Ok(ListCmd {
            client: client,
            config: cfg,
            output_format: *output_format,
            sort: sort,
            table_style: table_style,
        })
    }

    pub async fn get_zones(&self) -> Result<Option<ZoneList>, TdnsError> {
        let host = self.config.get_host();
        let url = format!(
            "{host}{API_LIST_ZONES_PATH}?token={}",
            self.config.get_token()
        );
        println!("Requesting zones from URL: {}", url);

        let body = match self.client.get_body(&url).await {
            Ok(body) => body,
            Err(error) => {
                return Err(self.make_http_error(error));
            }
        };
        println!("Response body: {}", body);

        let resp: ListZonesResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(error) => {
                return Err(self.make_json_error(error));
            }
        };

        if let Some(mut zone_list) = resp.zone_list {
            match self.sort {
                ZoneSortMode::AlphabeticalAscending => {
                    zone_list.zones.sort_by(|a, b| a.name.cmp(&b.name));
                }
                ZoneSortMode::AlphabeticalDescending => {
                    zone_list.zones.sort_by(|a, b| a.name.cmp(&b.name));
                    zone_list.zones.reverse();
                }
                _ => (),
            }
            Ok(Some(zone_list))
        } else {
            Ok(None)
        }
    }

    pub async fn execute(&self) -> Result<(), TdnsError> {
        let zones = self.get_zones().await?;
        if let Some(zone_list) = zones {
            if zone_list.zones.is_empty() {
                println!("No zones found");
                return Ok(());
            }
            match self.output_format {
                cli::OutputFormat::Json => {
                    let json = match serde_json::to_string_pretty(&zone_list) {
                        Ok(j) => j,
                        Err(error) => {
                            return Err(self.make_json_error(error));
                        }
                    };
                    println!("{}", json);
                }
                cli::OutputFormat::Table => {
                    // let table_style = Style::ascii_rounded()
                    //     .horizontals([(1, HorizontalLine::inherit(Style::ascii()).horizontal('-'))]);

                    for zone in zone_list.zones {
                        let mut zone_table = zone.to_table();
                        self.table_style.print_table(&mut zone_table);
                    }
                }
            }
        } else {
            println!("No zones found");
        }

        Ok(())
    }
}

impl<C: TdnsClient> TdnsErrorGenerator for ListCmd<C> {
    fn get_command_name(&self) -> &str {
        CMD_NAME
    }
    fn get_host(&self) -> &str {
        self.config.get_host()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "test-token";
    const HOST: &str = "test-host.example.com";

    #[tokio::test]
    async fn get_one_zone() {
        let mut mock_client = crate::client::MockTdnsClient::new();
        mock_client.expect_get_body().returning(|_| {
            Ok(r#"{
                "response": {
                    "zones": [
                        {
                            "name": "example.com",
                            "type": "Primary",
                            "internal": false,
                            "dnssecStatus": "Secure",
                            "soaSerial": 123456,
                            "lastModified": "2022-02-26T07:57:08.1842183Z",
                            "disabled": false
                        }
                    ]
                },
                "server": "ns.example.com",
                "status": "success"
            }"#
            .to_string())
        });

        // Create a config file
        let mut mock_cfg_mgr = config::MockConfigManager::new();
        mock_cfg_mgr
            .expect_read_config_file()
            .returning(|_| Ok(config::Config::new(HOST, TOKEN)));

        let list_cmd = ListCmd::create(
            mock_cfg_mgr,
            mock_client,
            "config.json",
            &cli::OutputFormat::Table,
            ZoneSortMode::Unsorted,
            TableStyles::Ascii,
        )
        .unwrap();

        let zones = list_cmd.get_zones().await.unwrap().unwrap();
        assert_eq!(zones.zones.len(), 1);

        let zone = &zones.zones[0];
        assert_eq!(zone.name, "example.com");
        assert_eq!(zone.zone_type, "Primary");
        assert_eq!(zone.internal, false);
    }

    #[tokio::test]
    async fn get_two_zones_sorted_alphabetically() {
        let mut mock_client = crate::client::MockTdnsClient::new();
        mock_client.expect_get_body().returning(|_| {
            Ok(r#"{
                "response": {
                    "zones": [
                        {
                            "name": "example.com",
                            "type": "Primary",
                            "internal": false,
                            "dnssecStatus": "Secure",
                            "soaSerial": 123456,
                            "lastModified": "2025-02-26T07:57:08.1842183Z",
                            "disabled": false
                        },
                        {
                            "name": "0.in-addr.arpa",
                            "type": "Primary",
                            "lastModified": "2026-01-14T07:47:55.3604008Z",
                            "disabled": false,
                            "soaSerial": 1,
                            "internal": true,
                            "dnssecStatus": "Unsigned",
                            "hasDnssecPrivateKeys": false
                        }
                    ]
                },
                "server": "ns.example.com",
                "status": "success"
            }"#
            .to_string())
        });

        // Create a config file
        let mut mock_cfg_mgr = config::MockConfigManager::new();
        mock_cfg_mgr
            .expect_read_config_file()
            .returning(|_| Ok(config::Config::new(HOST, TOKEN)));

        let list_cmd = ListCmd::create(
            mock_cfg_mgr,
            mock_client,
            "config.json",
            &cli::OutputFormat::Table,
            ZoneSortMode::AlphabeticalAscending,
            TableStyles::Ascii,
        )
        .unwrap();

        let zones = list_cmd.get_zones().await.unwrap().unwrap();
        assert_eq!(zones.zones.len(), 2);

        assert_eq!(zones.zones[0].name, "0.in-addr.arpa");
        assert_eq!(zones.zones[1].name, "example.com");
    }
}
