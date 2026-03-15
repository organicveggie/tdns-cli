use rstest::rstest;
use serde::{Deserialize, Serialize};
use std::{io::Cursor, path::PathBuf, rc::Rc};

use tdns::{config, run_cli};

#[derive(Debug, Deserialize, Serialize)]
struct AddRecordTestCase {
    name: String,
    zone: String,
    domain: String,
    token: String,
    ttl: Option<u32>,
    expiry_ttl: Option<u32>,
    overwrite: bool,
    comments: Option<String>,
    response: String,
    command: tdns::zone::add::RecordTypeCommand,
}

#[rstest]
#[tokio::test]
async fn test_add_record_toml(
    #[files("tests/fixtures/zone/record/add/*.toml")] testcase_path: PathBuf,
) {
    let file_content = std::fs::read_to_string(testcase_path.as_path()).unwrap();
    let test_case: AddRecordTestCase = match toml::from_str(&file_content) {
        Ok(tc) => tc,
        Err(error) => panic!("Failed to parse test case from TOML file: {}", error),
    };

    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let mock = server
        .mock("POST", tdns::zone::add::API_ADD_RECORD_PATH)
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(&test_case.response)
        .create_async()
        .await;

    // Create a config file
    let mut mock_cfg_mgr = config::MockConfigManager::new();
    mock_cfg_mgr
        .expect_read_config_file()
        .returning(move |_| Ok(config::Config::new(&url.clone(), &test_case.token)));

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new(true).unwrap();

    let cli_command = tdns::Command::Zone {
        zone: test_case.zone,
        zone_command: tdns::zone::Command::Add {
            domain: test_case.domain,
            ttl: test_case.ttl,
            overwrite: test_case.overwrite,
            comments: test_case.comments,
            expiry_ttl: test_case.expiry_ttl,
            add_command: test_case.command,
        },
    };

    let mut output_cursor = Cursor::new(Vec::new());
    let mut output = config::OutputTarget { w: &mut output_cursor };
    let mut app_config = config::ApplicationConfig {
        config_manager: Box::new(mock_cfg_mgr),
        tdns_client: Rc::new(client),
        output: &mut output,
    };

    run_cli(&mut app_config, "test-config.json", &cli_command).await;
    mock.assert();
}
