use std::{collections::HashMap, io::Cursor, rc::Rc};

use tdns::{config, run_cli, zone};

const TOKEN: &str = "test-add-token";

async fn mock_response(
    server: &mut mockito::Server,
    name: &str,
    record_type: zone::enums::ZoneRecordType,
    content: &str,
    extra: Option<HashMap<String, String>>,
) -> mockito::Mock {
    let mut record_data = HashMap::from([
        ("id".to_string(), "123".to_string()),
        ("name".to_string(), name.to_string()),
        ("type".to_string(), record_type.to_string()),
        ("content".to_string(), content.to_string()),
        ("ttl".to_string(), "3600".to_string()),
        ("lastModified".to_string(), "2025-02-26T07:57:08.1842183Z".to_string()),
    ]);
    if let Some(extra) = extra {
        for (key, value) in extra {
            record_data.insert(key, value);
        }
    }
    let record_json = serde_json::to_string(&record_data).unwrap();
    server
        .mock("POST", tdns::zone::add::API_ADD_RECORD_PATH)
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(format!(
            r#"{{
                "response": {{
                    "record": {record_json}
                }},
                "server": "ns.example.com",
                "status": "success"
        }}"#
        ))
        .create_async()
        .await
}

#[tokio::test]
async fn test_add_a_record() {
    // Define constants for the test
    const ADDRESS: &str = "192.168.1.17";
    const DOMAIN: &str = "ipv4host";
    const ZONE: &str = "example.com";

    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let mock =
        mock_response(&mut server, DOMAIN, zone::enums::ZoneRecordType::A, ADDRESS, None).await;

    // Create a config file
    let mut mock_cfg_mgr = config::MockConfigManager::new();
    mock_cfg_mgr
        .expect_read_config_file()
        .returning(move |_| Ok(config::Config::new(&url.clone(), TOKEN)));

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new(true).unwrap();

    let cli_command = tdns::Command::Zone {
        zone: ZONE.to_string(),
        zone_command: tdns::zone::Command::Add {
            domain: DOMAIN.to_string(),
            add_command: tdns::zone::add::RecordTypeCommand::A {
                address: ADDRESS.to_string(),
                ptr: false,
                ptr_zone: false,
                update_svcb_hints: false,
            },
            ttl: Some(3600),
            comments: Some("Test record".to_string()),
            overwrite: false,
            expiry_ttl: None,
        },
    };

    let mut output_cursor = Cursor::new(Vec::new());
    let mut output = config::OutputTarget{w: &mut output_cursor};
    let mut app_config = config::ApplicationConfig {
        config_manager: Box::new(mock_cfg_mgr),
        tdns_client: Rc::new(client),
        output: &mut output,
    };

    run_cli(&mut app_config, "test-config.json", &cli_command).await;
    mock.assert();
}

#[tokio::test]
async fn test_add_aaaa_record() {
    // Define constants for the test
    const ADDRESS: &str = "2001:db8::1";
    const DOMAIN: &str = "ipv6host";
    const ZONE: &str = "example.com";

    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let mock =
        mock_response(&mut server, DOMAIN, zone::enums::ZoneRecordType::AAAA, ADDRESS, None).await;

    // Create a config file
    let mut mock_cfg_mgr = config::MockConfigManager::new();
    mock_cfg_mgr
        .expect_read_config_file()
        .returning(move |_| Ok(config::Config::new(&url.clone(), TOKEN)));

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new(true).unwrap();

    let cli_command = tdns::Command::Zone {
        zone: ZONE.to_string(),
        zone_command: tdns::zone::Command::Add {
            domain: DOMAIN.to_string(),
            add_command: tdns::zone::add::RecordTypeCommand::AAAA {
                address: ADDRESS.to_string(),
                ptr: false,
                ptr_zone: false,
                update_svcb_hints: false,
            },
            ttl: None,
            comments: None,
            overwrite: false,
            expiry_ttl: None,
        },
    };

    let mut output_cursor = Cursor::new(Vec::new());
    let mut output = config::OutputTarget{w: &mut output_cursor};
    let mut app_config = config::ApplicationConfig {
        config_manager: Box::new(mock_cfg_mgr),
        tdns_client: Rc::new(client),
        output: &mut output,
    };

    run_cli(&mut app_config, "test-config.json", &cli_command).await;
    mock.assert();
}

#[tokio::test]
async fn test_add_cname_record() {
    // Define constants for the test
    const CNAME: &str = "host2.example.com";
    const DOMAIN: &str = "cnamehost";
    const ZONE: &str = "example.com";

    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let mock =
        mock_response(&mut server, DOMAIN, zone::enums::ZoneRecordType::CNAME, CNAME, None).await;

    // Create a config file
    let mut mock_cfg_mgr = config::MockConfigManager::new();
    mock_cfg_mgr
        .expect_read_config_file()
        .returning(move |_| Ok(config::Config::new(&url.clone(), TOKEN)));

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new(true).unwrap();

    let cli_command = tdns::Command::Zone {
        zone: ZONE.to_string(),
        zone_command: tdns::zone::Command::Add {
            domain: DOMAIN.to_string(),
            add_command: tdns::zone::add::RecordTypeCommand::CNAME { cname: CNAME.to_string() },
            ttl: None,
            comments: None,
            overwrite: false,
            expiry_ttl: None,
        },
    };

    let mut output_cursor = Cursor::new(Vec::new());
    let mut output = config::OutputTarget{w: &mut output_cursor};
    let mut app_config = config::ApplicationConfig {
        config_manager: Box::new(mock_cfg_mgr),
        tdns_client: Rc::new(client),
        output: &mut output,
    };

    run_cli(&mut app_config, "test-config.json", &cli_command).await;
    mock.assert();
}
