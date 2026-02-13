use serde::Deserialize;
use std::fmt;
use tabled::{Table, builder::Builder};

use crate::zones;

const UNUSED_INTERVAL: &str = "0s";
const UNUSED_TIMESTAMP: &str = "0001-01-01T00:00:00";

#[derive(Debug, Deserialize)]
#[serde(rename = "response")]
pub struct ListZoneRecordsResponse {
    #[serde(rename = "response")]
    pub records: RecordsList,
}

#[derive(Debug, Deserialize)]
pub struct RecordsList {
    pub zone: zones::Zone,
    pub records: Vec<ZoneRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDataA {
    #[serde(flatten)]
    ip_address: IpAddress,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDataAAAA {
    #[serde(flatten)]
    ip_address: IpAddress,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDataCNAME {
    cname: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDataNS {
    name_server: String,
}

#[derive(Debug, Deserialize, strum::Display)]
#[serde(tag = "type", content = "rData")]
pub enum RecordData {
    #[serde(rename_all = "camelCase")]
    A {
        #[serde(flatten)]
        ip_address: RecordDataA,
    },
    #[serde(rename_all = "camelCase")]
    AAAA {
        #[serde(flatten)]
        ip_address: RecordDataAAAA,
    },
    CNAME {
        #[serde(flatten)]
        cname: RecordDataCNAME,
    },
    #[serde(rename_all = "camelCase")]
    NS {
        #[serde(flatten)]
        name_server: RecordDataNS,
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

impl RecordData {
    pub fn value_summary(&self) -> String {
        match self {
            RecordData::A { ip_address } => format!("{}", ip_address.ip_address),
            RecordData::AAAA { ip_address } => format!("{}", ip_address.ip_address),
            RecordData::CNAME { cname } => format!("{}", cname.cname),
            RecordData::NS { name_server } => format!("{}", name_server.name_server),
            RecordData::SOA {
                primary_name_server,
                serial,
            } => format!("{} [{}]", primary_name_server, serial),
            RecordData::Unknown {
                record_type: _,
                r_data,
            } => {
                format!("{}", r_data.to_string())
            }
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpAddress {
    pub ip_address: String,
}

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ip_address)
    }
}

impl fmt::Display for ZoneRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        writeln!(
            f,
            "  {} {}",
            self.data.to_string(),
            self.data.value_summary()
        )
    }
}

impl ZoneRecord {
    pub fn to_detailed_table(&self) -> Table {
        let mut b = Builder::with_capacity(4, 2);
        b.push_record(["TTL", format!("{}", self.ttl).as_str()]);
        if let Some(ttl) = &self.ttl_string {
            b.push_record(["TTL", ttl]);
        }
        b.push_record(["Disabled", format!("{}", self.disabled).as_str()]);
        b.push_record(["DNSSec", &self.dnssec_status]);

        if let Some(last_used_on) = &self.last_used_on
            && last_used_on != UNUSED_TIMESTAMP
        {
            b.push_record(["Last Used On", last_used_on]);
        }
        if let Some(last_modified) = &self.last_modified {
            b.push_record(["Last Modified", last_modified]);
        }

        if let Some(expiry_ttl) = self.expiry_ttl
            && expiry_ttl > 0
        {
            b.push_record(["Expiry TTL", format!("{}", expiry_ttl).as_str()]);
        }
        if let Some(expiry_ttl) = &self.expiry_ttl_string
            && expiry_ttl != UNUSED_INTERVAL
        {
            b.push_record(["Expiry TTL", expiry_ttl]);
        }

        b.build()
    }
}
