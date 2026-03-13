use rstest::rstest;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, rc::Rc};
use tdns::{config, run_cli, zone};

const TOKEN: &str = "test-add-token";

fn make_mock_config(url: String, token: String) -> config::MockConfigManager {
    let mut mock = config::MockConfigManager::new();
    mock.expect_read_config_file()
        .returning(move |_| Ok(config::Config::from_strings(url.clone(), token.clone())));
    mock
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateRecordTestCase {
    name: String,
    zone: String,
    domain: String,
    ttl: Option<u32>,
    comments: Option<String>,
    expiry_ttl: Option<u32>,
    command: zone::update::RecordTypeCommand,
    mock_response: Option<serde_json::Value>,
}

#[rstest]
#[tokio::test]
async fn test_update_record(#[files("tests/fixtures/zone/update/update*.json")] path: PathBuf) {
    use std::io::Cursor;

    let file_content = std::fs::read_to_string(path).unwrap();
    let test_case: UpdateRecordTestCase = match serde_json::from_str(&file_content) {
        Ok(tc) => tc,
        Err(e) => panic!("Failed to parse test case from JSON: {}", e),
    };

    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Create a config file
    let mock_cfg_mgr = make_mock_config(url, TOKEN.to_string());

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new(true).unwrap();

    // Setup the mock response based on the test case
    let body = match test_case.mock_response {
        Some(resp) => serde_json::to_string(&resp).unwrap(),
        None => "".to_string(),
    };
    let mock = server
        .mock("POST", zone::update::API_UPDATE_RECORD_PATH)
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let cli_command = tdns::Command::Zone {
        zone: test_case.zone.to_string(),
        zone_command: tdns::zone::Command::Update {
            comments: test_case.comments.clone(),
            domain: test_case.domain.to_string(),
            expiry_ttl: test_case.expiry_ttl,
            ttl: test_case.ttl,
            update_command: test_case.command,
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
