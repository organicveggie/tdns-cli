use std::{cell::RefCell, collections::HashMap, rc::Rc};

use tdns::{
    config, run_cli,
    zone::{enums, get_records::ZoneRecordDetailLevel},
};

const TOKEN: &str = "test-add-token";

fn make_zone_json(name: &str) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "type": "Primary",
        "internal": false,
        "dnssecStatus": "SignedWithNSEC3",
        "disabled": false
    })
}

fn make_record_json(
    name: &str,
    zone_type: enums::ZoneRecordType,
    rdata: HashMap<String, String>,
) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "type": zone_type.to_string(),
        "ttl": 3600,
        "disabled": false,
        "dnssecStatus": "Unknown",
        "rData": rdata
    })
}

async fn make_mock_reponse(
    server: &mut mockito::Server,
    zone_name: &str,
    records: Vec<serde_json::Value>,
) -> mockito::Mock {
    let json_response = serde_json::json!({
        "response": {
            "zone": make_zone_json(zone_name),
            "records": records
        },
        "status": "ok"
    });

    server
        .mock("GET", tdns::zone::get_records::API_GET_RECORDS_PATH)
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(json_response.to_string())
        .create_async()
        .await
}

#[tokio::test]
async fn test_get_all_zone_records() {
    const ZONE: &str = "example.com";

    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let mock = make_mock_reponse(
        &mut server,
        ZONE,
        vec![
            make_record_json(
                ZONE,
                enums::ZoneRecordType::A,
                HashMap::from([("ipAddress".to_string(), "1.1.1.1".to_string())]),
            ),
            make_record_json(
                ZONE,
                enums::ZoneRecordType::NS,
                HashMap::from([("nameserver".to_string(), "ns1.example.com".to_string())]),
            ),
        ],
    )
    .await;

    // Create a config file
    let mut mock_cfg_mgr = config::MockConfigManager::new();
    mock_cfg_mgr
        .expect_read_config_file()
        .returning(move |_| Ok(config::Config::new(&url.clone(), TOKEN)));

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new(true).unwrap();

    let cli_command = tdns::Command::Zone {
        zone: ZONE.to_string(),
        zone_command: tdns::zone::Command::List {
            domain: None,
            detail: ZoneRecordDetailLevel::Summary,
            table_style: tdns::tables::TableStyles::Ascii,
        },
    };

    let writer = Rc::new(RefCell::new(Vec::<u8>::new()));
    let app_config = config::ApplicationConfig {
        config_manager: Box::new(mock_cfg_mgr),
        tdns_client: Rc::new(client),
        output: config::OutputTarget::IoWrite { writer: writer.clone() },
    };

    run_cli(&app_config, "test-config.json", &cli_command).await;
    mock.assert();
}

#[tokio::test]
async fn test_get_one_zone_record() {
    const ZONE: &str = "example.com";
    let hostname = format!("host1.{}", ZONE);

    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let mock = make_mock_reponse(
        &mut server,
        ZONE,
        vec![make_record_json(
            &hostname,
            enums::ZoneRecordType::A,
            HashMap::from([("ipAddress".to_string(), "1.2.3.4".to_string())]),
        )],
    )
    .await;

    // Create a config file
    let mut mock_cfg_mgr = config::MockConfigManager::new();
    mock_cfg_mgr
        .expect_read_config_file()
        .returning(move |_| Ok(config::Config::new(&url.clone(), TOKEN)));

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new(true).unwrap();

    let cli_command = tdns::Command::Zone {
        zone: ZONE.to_string(),
        zone_command: tdns::zone::Command::List {
            domain: Some(hostname),
            detail: ZoneRecordDetailLevel::Summary,
            table_style: tdns::tables::TableStyles::Ascii,
        },
    };

    let writer = Rc::new(RefCell::new(Vec::<u8>::new()));
    let app_config = config::ApplicationConfig {
        config_manager: Box::new(mock_cfg_mgr),
        tdns_client: Rc::new(client),
        output: config::OutputTarget::IoWrite { writer: writer.clone() },
    };

    run_cli(&app_config, "test-config.json", &cli_command).await;
    mock.assert();
}
