use tdns::{config, run_cli};

const TOKEN: &str = "test-token";

#[tokio::test]
async fn test_two_zones_sorted() {
    // Request a new server from the pool
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    println!("Mock server running at: {}", url);

    // Create mock response for the /api/zones/list endpoint
    let mock = server
        .mock("GET", tdns::zones::API_LIST_ZONES_PATH)
        .match_query(format!("token={}", TOKEN).as_str())
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{
                "response": {
                    "zones": [
                        {
                            "name": "example.com",
                            "type": "Primary",
                            "internal": false,
                            "dnssecStatus": "Secure",
                            "soaSerial": 123456,
                            "lastModified": "2025-02-26T07:57:08.1842183Z",
                            "disabled": false
                        },
                        {
                            "name": "0.in-addr.arpa",
                            "type": "Primary",
                            "lastModified": "2026-01-14T07:47:55.3604008Z",
                            "disabled": false,
                            "soaSerial": 1,
                            "internal": true,
                            "dnssecStatus": "Unsigned",
                            "hasDnssecPrivateKeys": false
                        }
                    ]
                },
                "server": "ns.example.com",
                "status": "success"
            }"#,
        )
        .create_async()
        .await;

    // Create a config file
    let mut mock_cfg_mgr = config::MockConfigManager::new();
    mock_cfg_mgr
        .expect_read_config_file()
        .returning(move |_| Ok(config::Config::new(&url.clone(), TOKEN)));

    // Create a TdnsClient that points to the mock server
    let client = tdns::client::TdnsHttpClient::new().unwrap();

    let cli_command = tdns::Command::List {
        sort_order: tdns::cli::SortOrder::Ascending,
        output_format: tdns::cli::OutputFormat::Json,
        table_style: tdns::tables::TableStyles::Ascii,
    };

    run_cli(mock_cfg_mgr, client, "test-config.json", &cli_command).await;

    mock.assert();
}
