use clap::Subcommand;
use serde::Deserialize;

use crate::client::QueryBuilder;
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
        let mut qb = QueryBuilder::new()
            .add_param("token", &cfg.get_token())
            .add_param("domain", &domain_name)
            .add_param("zone", &zone)
            .add_param("type", &self.to_string());

        if overwrite {
            qb = qb.add_param("overwrite", "true");
        }
        if let Some(comments) = comments {
            qb = qb.add_param("comments", &comments);
        }
        if let Some(ttl) = ttl {
            qb = qb.add_param("ttl", &ttl.to_string());
        }
        if let Some(expiry_ttl) = expiry_ttl {
            qb = qb.add_param("expiryTtl", &expiry_ttl.to_string());
        }

        match self {
            RecordTypeCommand::A { address, ptr, ptr_zone, update_svcb_hints } => {
                let addr_qb = make_address_params(address, *ptr, *ptr_zone, *update_svcb_hints);
                qb = qb.merge(addr_qb);
            }
            RecordTypeCommand::AAAA { address, ptr, ptr_zone, update_svcb_hints } => {
                let addr_qb = make_address_params(address, *ptr, *ptr_zone, *update_svcb_hints);
                qb = qb.merge(addr_qb);
            }
            RecordTypeCommand::CNAME { cname } => {
                qb = qb.add_param("cname", cname);
            }
            RecordTypeCommand::TXT { text, split_text } => {
                qb = qb.add_param("text", text);
                if *split_text {
                    qb = qb.add_param("splitText", "true");
                }
            }
            RecordTypeCommand::MX { exchange, preference } => {
                qb = qb.add_param("exchange", exchange);
                if let Some(pref) = preference {
                    qb = qb.add_param("preference", &pref.to_string());
                }
            }
            RecordTypeCommand::NS { name_server, glue } => {
                qb = qb.add_param("nameServer", name_server);
                if let Some(glue_addr) = glue {
                    qb = qb.add_param("glue", glue_addr);
                }
            }
            RecordTypeCommand::PTR { ptr_name } => {
                qb = qb.add_param("ptrName", ptr_name);
            }
            _ => { /* Other record types can be handled here */ }
        }

        let url = format!("{host}/api/zones/records/add?{}", qb.build());

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

fn make_address_params(
    address: &str,
    ptr: bool,
    ptr_zone: bool,
    update_svcb_hints: bool,
) -> QueryBuilder {
    let mut qb = QueryBuilder::new().add_param("address", address);
    if ptr {
        qb = qb.add_param("ptr", "true");
    }
    if ptr_zone {
        qb = qb.add_param("ptrZone", "true");
    }
    if update_svcb_hints {
        qb = qb.add_param("updateSvcbHints", "true");
    }
    qb
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
    fn make_address_params_test() {
        #[rustfmt::skip]
        let cases = vec![
            ("1.2.3.4", false, false, false, "address=1.2.3.4"),
            ("2.3.4.5", true, false, false, "address=2.3.4.5&ptr=true"),
            ("3.4.5.6", false, true, false, "address=3.4.5.6&ptrZone=true"),
            ("4.5.6.7", true, true, false, "address=4.5.6.7&ptr=true&ptrZone=true"),
            ("5.6.7.8", false, false, true, "address=5.6.7.8&updateSvcbHints=true"),
            ("6.7.8.9", true, false, true, "address=6.7.8.9&ptr=true&updateSvcbHints=true"),
            ("7.8.9.0", false, true, true, "address=7.8.9.0&ptrZone=true&updateSvcbHints=true"),
            ("8.9.0.1", true, true, true, "address=8.9.0.1&ptr=true&ptrZone=true&updateSvcbHints=true"),
            
        ];
        for (address, ptr, ptr_zone, update_svcb_hints, expected) in cases {
            assert_eq!(
                make_address_params(address, ptr, ptr_zone, update_svcb_hints).build(),
                expected
            );
        }
    }
}
