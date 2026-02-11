use clap::Subcommand;
use serde::Deserialize;

use crate::{config, errors};

pub const CMD_NAME: &str = "Add Zone Record";

#[derive(Debug, strum::Display, strum::EnumString, Subcommand)]
pub enum RecordTypeCommand {
    A {
        #[arg(help = "IPv4 address for the A record")]
        address: String,
        #[arg(
            long = "ptr",
            default_value_t = false,
            help = "Create a reverse PTR record for the IP address"
        )]
        ptr: bool,
        #[arg(
            long = "ptr_zone",
            default_value_t = false,
            help = "Create a reverse zone for the PTR record"
        )]
        ptr_zone: bool,
        #[arg(
            long = "update_svcb_hints",
            default_value_t = false,
            help = "Update SVCB hints in the zone for the current record"
        )]
        update_svcb_hints: bool,
    },
    AAAA {
        #[arg(help = "IPv6 address for the AAAA record")]
        address: String,
        #[arg(
            long = "ptr",
            default_value_t = false,
            help = "Create a reverse PTR record for the IP address"
        )]
        ptr: bool,
        #[arg(
            long = "ptr_zone",
            default_value_t = false,
            help = "Create a reverse zone for the PTR record"
        )]
        ptr_zone: bool,
        #[arg(
            long = "update_svcb_hints",
            default_value_t = false,
            help = "Update SVCB hints in the zone for the current record"
        )]
        update_svcb_hints: bool,
    },
    ANAME,
    APP,
    CAA,
    CNAME {
        #[arg(help = "Canonical domain name")]
        cname: String,
    },
    DNAME,
    DS,
    FWD,
    HTTPS,
    MX {
        #[arg(help = "Exchange domain name")]
        exchange: String,
        #[arg(help = "Preference value for the MX record")]
        preference: Option<u16>,
    },
    NS {
        #[arg(help = "Name server domain name")]
        name_server: String,
        #[arg(help = "Glue address for the name server in the NS record")]
        glue: Option<String>,
    },
    PTR {
        #[arg(help = "PTR domain name")]
        ptr_name: String,
    },
    SRV,
    SSHFP,
    SVCB,
    TLSA,
    TXT {
        #[arg(help = "Text value for the TXT record")]
        text: String,
        #[arg(
            long = "split_text",
            help = "Split text into multiple strings when adding TXT record",
            default_value_t = false
        )]
        split_text: bool,
    },
    URI,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "response")]
pub struct AddRecordResponse {
    status: String,
    response: Option<serde_json::Value>,
}

impl RecordTypeCommand {
    pub async fn run(
        &self,
        app_config: &config::ApplicationConfig,
        config_file_name: &str,
        zone: String,
        domain: String,
        overwrite: bool,
        comments: Option<String>,
        ttl: Option<u32>,
        expiry_ttl: Option<u32>,
    ) -> Result<(), errors::TdnsError> {
        let domain_name = make_domain_name(&domain, &zone);

        let cfg = match app_config.config_manager.read_config_file(config_file_name) {
            Ok(c) => c,
            Err(error) => {
                return Err(errors::make_config_error(CMD_NAME, error));
            }
        };

        let host = cfg.get_host();
        let mut url = format!(
            "{host}/api/zones/records/add?token={}&domain={}&zone={}&type={}",
            cfg.get_token(),
            domain_name,
            zone,
            self.to_string()
        );

        match self {
            RecordTypeCommand::A { address, ptr, ptr_zone, update_svcb_hints } => {
                url = add_address_params(&url, address, *ptr, *ptr_zone, *update_svcb_hints);
            }
            RecordTypeCommand::AAAA { address, ptr, ptr_zone, update_svcb_hints } => {
                url = add_address_params(&url, address, *ptr, *ptr_zone, *update_svcb_hints);
            }
            RecordTypeCommand::CNAME { cname } => {
                url = format!("{}&cname={}", url, cname);
            }
            RecordTypeCommand::TXT { text, split_text } => {
                url = if *split_text {
                    format!("{}&text={}&splitText=true", url, text)
                } else {
                    format!("{}&text={}", url, text)
                };
            }
            RecordTypeCommand::MX { exchange, preference } => {
                url = format!("{}&exchange={}", url, exchange);
                if let Some(pref) = preference {
                    url = format!("{}&preference={}", url, pref);
                }
            }
            RecordTypeCommand::NS { name_server, glue } => {
                url = format!("{}&nameServer={}", url, name_server);
                if let Some(glue_addr) = glue {
                    url = format!("{}&glue={}", url, glue_addr);
                }
            }
            RecordTypeCommand::PTR { ptr_name } => {
                url = format!("{}&ptrName={}", url, ptr_name);
            }
            _ => { /* Other record types can be handled here */ }
        }

        let body = match app_config.tdns_client.get_body(&url).await {
            Ok(b) => b,
            Err(error) => {
                return Err(errors::make_http_error(CMD_NAME, host, error));
            }
        };

        let resp: AddRecordResponse = match serde_json::from_str(&body) {
            Ok(resp) => resp,
            Err(error) => {
                return Err(errors::make_json_error(CMD_NAME, error));
            }
        };

        println!("Response status: {}", resp.status);
        if let Some(resp) = resp.response {
            println!("Response data: {}", resp);
        }

        Ok(())
    }
}

fn make_domain_name(domain: &str, zone: &str) -> String {
    if domain.ends_with(zone) {
        return domain.to_string();
    }

    if domain.ends_with('.') {
        format!("{}{}", domain, zone)
    } else {
        format!("{}.{}", domain, zone)
    }
}

fn add_address_params(
    url: &str,
    address: &str,
    ptr: bool,
    ptr_zone: bool,
    update_svcb_hints: bool,
) -> String {
    let mut updated_url = format!("{}&address={}", url, address);
    if ptr {
        updated_url = format!("{}&ptr=true", updated_url);
    }
    if ptr_zone {
        updated_url = format!("{}&ptrZone=true", updated_url);
    }
    if update_svcb_hints {
        updated_url = format!("{}&updateSvcbHints=true", updated_url);
    }
    updated_url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_domain_name_test() {
        let cases = vec![
            ("www", "example.com", "www.example.com"),
            ("www.example.com", "example.com", "www.example.com"),
            ("www.", "example.com", "www.example.com"),
            ("www.sub", "example.com", "www.sub.example.com"),
            ("www.sub.", "example.com", "www.sub.example.com"),
        ];
        for (domain, zone, expected) in cases {
            assert_eq!(make_domain_name(domain, zone), expected);
        }
    }

    #[test]
    fn add_address_params_test() {
        let base_url = "http://example.com/api?token=abc";

        #[rustfmt::skip]
        let cases = vec![
            ("1.2.3.4", false, false, false, "http://example.com/api?token=abc&address=1.2.3.4"),
            ("2.3.4.5", true, false, false, "http://example.com/api?token=abc&address=2.3.4.5&ptr=true"),
            ("3.4.5.6", false, true, false, "http://example.com/api?token=abc&address=3.4.5.6&ptrZone=true"),
            ("4.5.6.7", true, true, false, "http://example.com/api?token=abc&address=4.5.6.7&ptr=true&ptrZone=true"),
            ("5.6.7.8", false, false, true, "http://example.com/api?token=abc&address=5.6.7.8&updateSvcbHints=true"),
            ("6.7.8.9", true, false, true, "http://example.com/api?token=abc&address=6.7.8.9&ptr=true&updateSvcbHints=true"),
            ("7.8.9.0", false, true, true, "http://example.com/api?token=abc&address=7.8.9.0&ptrZone=true&updateSvcbHints=true"),
            ("8.9.0.1", true, true, true, "http://example.com/api?token=abc&address=8.9.0.1&ptr=true&ptrZone=true&updateSvcbHints=true"),
            
        ];
        for (address, ptr, ptr_zone, update_svcb_hints, expected) in cases {
            assert_eq!(
                add_address_params(base_url, address, ptr, ptr_zone, update_svcb_hints),
                expected
            );
        }
    }
}
