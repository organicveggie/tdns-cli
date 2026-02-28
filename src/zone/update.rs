use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::client::QueryBuilder;
use crate::{config, errors, zone::helpers};

pub const CMD_NAME: &str = "Update Zone Record";
pub const API_UPDATE_RECORD_PATH: &str = "/api/zones/records/update";

#[derive(Debug, Deserialize, strum::Display, Serialize, strum::EnumString, Subcommand)]
pub enum RecordTypeCommand {
    A {
        #[arg(help = "IPv4 address for the A record")]
        address: Option<String>,

        #[arg(long, help = "Create a reverse PTR record for the IP address")]
        ptr: Option<bool>,

        #[arg(long, help = "Create a reverse zone for the PTR record")]
        ptr_zone: Option<bool>,

        #[arg(
            long,
            help = "Update SVCB hints in the zone for the current record"
        )]
        update_svcb_hints: Option<bool>,
    },
    AAAA {
        #[arg(help = "IPv6 address for the AAAA record")]
        address: Option<String>,

        #[arg(long, help = "Create a reverse PTR record for the IP address")]
        ptr: Option<bool>,

        #[arg(long, help = "Create a reverse zone for the PTR record")]
        ptr_zone: Option<bool>,

        #[arg(
            long,
            help = "Update SVCB hints in the zone for the current record"
        )]
        update_svcb_hints: Option<bool>,
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
    SOA {
        #[arg(short, long, help = "Primary name server for the SOA record")]
        primary_ns: String,
        #[arg(long, help = "Responsible person for the SOA record")]
        responsible_person: String,
        #[arg(short, long, help = "Serial number for the SOA record")]
        serial: u32,
        #[arg(
            long,
            help = "Time in seconds before a secondary server should check the SOA for changes"
        )]
        refresh: u32,
        #[arg(
            long,
            help = "Time in seconds a secondary server should wait to retry a failed zone transfer"
        )]
        retry: u32,
        #[arg(
            short = 'x',
            long,
            help = "Time in seconds a secondary server will keep using cached data if the master is unreachable"
        )]
        expire: u32,
        #[arg(short, long, help = "Minimum TTL for resource records in the zone")]
        minimum: u32,
        #[arg(
            long,
            help = "Set value to true to enable using date scheme for SOA serial. This optional parameter is used only with Primary, Forwarder, and Catalog zones."
        )]
        use_serial_date_scheme: bool,
    },
    SRV,
    SSHFP,
    SVCB,
    TLSA,
    TXT {
        #[arg(help = "Text value for the TXT record")]
        text: Option<String>,

        #[arg(
            long,
            help = "Split text into multiple strings when adding TXT record"
        )]
        split_text: Option<bool>,
    },
    URI,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "response")]
pub struct UpdateRecordResponse {
    status: String,
    response: Option<serde_json::Value>,
}

impl RecordTypeCommand {
    pub async fn run<'a, T>(
        &self,
        app_config: &mut config::ApplicationConfig<'a, T>,
        config_file_name: &str,
        zone: String,
        domain: String,
        comments: Option<String>,
        ttl: Option<u32>,
        expiry_ttl: Option<u32>,
    ) -> Result<(), errors::TdnsError> where T: std::io::Write {
        let cfg = match app_config.config_manager.read_config_file(config_file_name) {
            Ok(c) => c,
            Err(error) => {
                return Err(errors::make_config_error(CMD_NAME, error));
            }
        };

        let domain_name = helpers::make_domain_name(&domain, &zone);

        let query_params =
            self.make_query_params(&cfg, &domain_name, &zone, comments, ttl, expiry_ttl);

        let host = cfg.get_host();
        let url = format!("{host}{API_UPDATE_RECORD_PATH}");

        let body = match app_config.tdns_client.post_body(&url, &None, &Some(query_params)).await {
            Ok(b) => b,
            Err(error) => {
                return Err(errors::make_http_error(CMD_NAME, host, error));
            }
        };

        let resp: UpdateRecordResponse = match serde_json::from_str(&body) {
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

    fn make_query_params(
        &self,
        cfg: &config::Config,
        domain_name: &str,
        zone: &str,
        comments: Option<String>,
        ttl: Option<u32>,
        expiry_ttl: Option<u32>,
    ) -> QueryBuilder {
        let mut qb = QueryBuilder::new()
            .add_param("token", &cfg.get_token())
            .add_param("domain", domain_name)
            .add_param("zone", zone)
            .add_param("type", &self.to_string());
        if let Some(comments) = comments {
            qb = qb.add_param("comments", &comments);
        }
        if let Some(ttl) = ttl {
            qb = qb.add_param("ttl", &ttl.to_string());
        }
        if let Some(expiry_ttl) = expiry_ttl {
            qb = qb.add_param("expiry_ttl", &expiry_ttl.to_string());
        }

        let extra_params = self.make_record_type_params();
        qb = qb.merge(extra_params);
        qb
    }

    fn make_record_type_params(&self) -> QueryBuilder {
        match self {
            RecordTypeCommand::A { address, ptr, ptr_zone, update_svcb_hints } => {
                make_address_params(address.clone(), *ptr, *ptr_zone, *update_svcb_hints)
            }
            RecordTypeCommand::AAAA { address, ptr, ptr_zone, update_svcb_hints } => {
                make_address_params(address.clone(), *ptr, *ptr_zone, *update_svcb_hints)
            }
            RecordTypeCommand::CNAME { cname } => QueryBuilder::new().add_param("cname", cname),
            RecordTypeCommand::MX { exchange, preference } => {
                let mut qb = QueryBuilder::new().add_param("exchange", exchange);
                if let Some(preference) = preference {
                    qb = qb.add_param("preference", &preference.to_string());
                }
                qb
            }
            RecordTypeCommand::NS { name_server, glue } => {
                let mut qb = QueryBuilder::new().add_param("name_server", name_server);
                if let Some(glue) = glue {
                    qb = qb.add_param("glue", glue);
                }
                qb
            }
            RecordTypeCommand::PTR { ptr_name } => {
                QueryBuilder::new().add_param("ptr_name", ptr_name)
            }
            RecordTypeCommand::SOA {
                primary_ns,
                responsible_person,
                serial,
                refresh,
                retry,
                expire,
                minimum,
                use_serial_date_scheme,
            } => {
                return QueryBuilder::new()
                    .add_param("primaryNameServer", primary_ns)
                    .add_param("responsiblePerson", responsible_person)
                    .add_param("serial", &serial.to_string())
                    .add_param("refresh", &refresh.to_string())
                    .add_param("retry", &retry.to_string())
                    .add_param("expire", &expire.to_string())
                    .add_param("minimum", &minimum.to_string())
                    .add_param("useSerialDateScheme", &use_serial_date_scheme.to_string());
            }
            RecordTypeCommand::TXT { text, split_text } => {
                let mut qb = QueryBuilder::new();
                if let Some(text) = text {
                    qb = qb.add_param("text", text);
                }
                if let Some(split_text) = split_text {
                    qb = qb.add_param("split_text", &split_text.to_string());
                }
                qb
            }
            _ => {
                /* Other record types can be handled here */
                QueryBuilder::new()
            }
        }
    }
}

fn make_address_params(
    address: Option<String>,
    ptr: Option<bool>,
    ptr_zone: Option<bool>,
    update_svcb_hints: Option<bool>,
) -> QueryBuilder {
    let mut qb = QueryBuilder::new();
    if let Some(address) = address {
        qb = qb.add_param("address", &address);
    }
    if let Some(ptr) = ptr {
        qb = qb.add_param("ptr", &ptr.to_string());
    }
    if let Some(ptr_zone) = ptr_zone {
        qb = qb.add_param("ptr_zone", &ptr_zone.to_string());
    }
    if let Some(update_svcb_hints) = update_svcb_hints {
        qb = qb.add_param("update_svcb_hints", &update_svcb_hints.to_string());
    }
    qb
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde::Serialize;
    use std::path::PathBuf;

    use super::*;

    #[rstest]
    #[case(Some("1.2.3.4".to_string()), None, None, None, vec![("address", "1.2.3.4")])]
    #[case(None, Some(true), None, None, vec![("ptr", "true")])]
    #[case(None, None, Some(true), None, vec![("ptr_zone", "true")])]
    #[case(None, None, None, Some(true), vec![("update_svcb_hints", "true")])]
    #[case(Some("1.2.3.4".to_string()), Some(true), None, None, vec![("address", "1.2.3.4"), ("ptr", "true")])]
    fn make_address_params_test_v2(
        #[case] addr: Option<String>,
        #[case] ptr: Option<bool>,
        #[case] ptr_zone: Option<bool>,
        #[case] update_svcb_hints: Option<bool>,
        #[case] expected_params: Vec<(&str, &str)>,
    ) {
        let expected = QueryBuilder::from(expected_params);
        assert_eq!(make_address_params(addr, ptr, ptr_zone, update_svcb_hints), expected);
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct QueryParamsExtraTestOptions {
        pub ttl: Option<u32>,
        pub comments: Option<String>,
        pub expiry_ttl: Option<u32>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct QueryParamsTestCase {
        domain: String,
        record_type_command: RecordTypeCommand,
        extra_options: Option<QueryParamsExtraTestOptions>,
        expected_params: QueryBuilder,
    }

    #[rstest]
    fn make_query_params_test(
        #[files("tests/fixtures/zone/update/make_query_params_*.toml")] path: PathBuf,
    ) {
        const TOKEN: &str = "token1";
        const ZONE: &str = "example.com";
        let cfg = config::Config::new("host1", TOKEN);

        let file_content = std::fs::read_to_string(path).unwrap();
        let test_case: QueryParamsTestCase = toml::from_str(&file_content).unwrap();
        let extras = match test_case.extra_options.as_ref() {
            Some(options) => options,
            None => &QueryParamsExtraTestOptions { ttl: None, comments: None, expiry_ttl: None },
        };

        let domain_name = helpers::make_domain_name(&test_case.domain, ZONE);
        let qb = test_case.record_type_command.make_query_params(
            &cfg,
            &domain_name,
            ZONE,
            extras.comments.clone(),
            extras.ttl,
            extras.expiry_ttl,
        );
        assert_eq!(qb, test_case.expected_params);
    }
}
