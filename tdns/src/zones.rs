use serde::{Deserialize, Serialize};
use std::fmt;
use tabled::Table;
use tabled::builder::Builder;
use tabled::settings::Panel;

use crate::client::TdnsClient;
use crate::config;
use crate::errors::{TdnsError, TdnsErrorGenerator};
use crate::tables::TableStyles;

const CMD_NAME: &str = "List Zones";

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

pub struct ListCmd<C: TdnsClient> {
    client: C,
    config: config::Config,
    sort: ZoneSortMode,
    table_style: TableStyles,
}

impl<C: TdnsClient> ListCmd<C> {
    pub fn create(
        client: C,
        config_file: &str,
        sort: ZoneSortMode,
        table_style: TableStyles,
    ) -> Result<ListCmd<C>, config::ConfigFileError> {
        let cfg = config::read_config_file(config_file)?;
        Ok(ListCmd {
            client: client,
            config: cfg,
            sort: sort,
            table_style: table_style,
        })
    }

    async fn get_zones(&self) -> Result<Option<ZoneList>, TdnsError> {
        let host = self.config.get_host();
        let url = format!("{host}/api/zones/list?token={}", self.config.get_token());

        let body = match self.client.get_body(&url).await {
            Ok(body) => body,
            Err(error) => {
                return Err(self.make_http_error(error));
            }
        };

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
            } else {
                // let table_style = Style::ascii_rounded()
                //     .horizontals([(1, HorizontalLine::inherit(Style::ascii()).horizontal('-'))]);

                for zone in zone_list.zones {
                    let mut zone_table = zone.to_table();
                    self.table_style.print_table(&mut zone_table);
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
