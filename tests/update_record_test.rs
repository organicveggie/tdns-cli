use rstest::rstest;
use serde::{Deserialize, Serialize};
use std::{io::Cursor, path::PathBuf, rc::Rc};
use tdns::{config, run_cli, zone};

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
    response: String,
}

#[rstest]
#[tokio::test]
async fn test_update_record_toml(
    #[files("tests/fixtures/zone/record/update/*.toml")] testcase_path: PathBuf,
) {
    const TOKEN: &str = "test-add-token";

    let file_content = std::fs::read_to_string(testcase_path.as_path()).unwrap();
    let test_case: UpdateRecordTestCase = match toml::from_str(&file_content) {
        Ok(tc) => tc,
        Err(error) => panic!("Failed to parse test case from TOML file: {}", error),
    };

    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // Create a config file
    let mock_cfg_mgr = make_mock_config(url, TOKEN.to_string());

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new(true).unwrap();

    // Setup the mock response based on the test case
    let mock = server
        .mock("POST", zone::update::API_UPDATE_RECORD_PATH)
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(test_case.response)
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

    let mut output_cursor = Cursor::new(Vec::<u8>::new());
    let mut output = config::OutputTarget { w: &mut output_cursor };
    let mut app_config = config::ApplicationConfig {
        config_manager: Box::new(mock_cfg_mgr),
        tdns_client: Rc::new(client),
        output: &mut output,
    };

    run_cli(&mut app_config, "test-config.json", &cli_command).await;

    mock.assert();
}
