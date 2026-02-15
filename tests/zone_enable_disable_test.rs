use std::cell::RefCell;
use std::rc::Rc;

use tdns::{config, run_cli, zone};

const TOKEN: &str = "test-add-token";

enum TestMode {
    Disable,
    Enable,
}

async fn mock_response(
    server: &mut mockito::Server,
    mode: TestMode,
    status: &str,
) -> mockito::Mock {
    let path = match mode {
        TestMode::Disable => zone::enable::API_DISABLE_ZONE_PATH,
        TestMode::Enable => zone::enable::API_ENABLE_ZONE_PATH,
    };
    server
        .mock("GET", path)
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(format!(
            r#"{{
                "status": "{status}"
            }}"#
        ))
        .create_async()
        .await
}

#[tokio::test]
async fn test_disable_zone() {
    // Define constants for the test
    const ZONE: &str = "example.com";

    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let mock = mock_response(&mut server, TestMode::Disable, "ok").await;

    // Create a config file
    let mut mock_cfg_mgr = config::MockConfigManager::new();
    mock_cfg_mgr
        .expect_read_config_file()
        .returning(move |_| Ok(config::Config::new(&url.clone(), TOKEN)));

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new().unwrap();
    let cli_command =
        tdns::Command::Zone { zone: ZONE.to_string(), zone_command: tdns::zone::Command::Disable };

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
async fn test_ensable_zone() {
    // Define constants for the test
    const ZONE: &str = "example.com";

    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let mock = mock_response(&mut server, TestMode::Enable, "ok").await;

    // Create a config file
    let mut mock_cfg_mgr = config::MockConfigManager::new();
    mock_cfg_mgr
        .expect_read_config_file()
        .returning(move |_| Ok(config::Config::new(&url.clone(), TOKEN)));

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new().unwrap();
    let cli_command =
        tdns::Command::Zone { zone: ZONE.to_string(), zone_command: tdns::zone::Command::Enable };

    let writer = Rc::new(RefCell::new(Vec::<u8>::new()));
    let app_config = config::ApplicationConfig {
        config_manager: Box::new(mock_cfg_mgr),
        tdns_client: Rc::new(client),
        output: config::OutputTarget::IoWrite { writer: writer.clone() },
    };

    run_cli(&app_config, "test-config.json", &cli_command).await;
    mock.assert();
}
