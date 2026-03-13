pub const CMD_NAME_ENABLE: &str = "Enable Zone";
pub const CMD_NAME_DISABLE: &str = "Disable Zone";

pub const API_ENABLE_ZONE_PATH: &str = "/api/zones/enable";
pub const API_DISABLE_ZONE_PATH: &str = "/api/zones/disable";

use serde::Deserialize;

use crate::client::QueryBuilder;
use crate::config;
use crate::errors;

pub enum Mode {
    Enable,
    Disable,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    status: String,
}

pub async fn run<'a, T>(
    app_config: &mut config::ApplicationConfig<'a, T>,
    config_file_name: &str,
    zone: String,
    mode: Mode,
) -> Result<(), errors::TdnsError> where T: std::io::Write {
    let command_name = match mode {
        Mode::Enable => CMD_NAME_ENABLE,
        Mode::Disable => CMD_NAME_DISABLE,
    };

    let cfg = match app_config.config_manager.read_config_file(config_file_name) {
        Ok(c) => c,
        Err(error) => {
            return Err(errors::make_config_error(command_name, error));
        }
    };
    let api_path = match mode {
        Mode::Enable => API_ENABLE_ZONE_PATH,
        Mode::Disable => API_DISABLE_ZONE_PATH,
    };

    let host = cfg.get_host();
    let url = format!("{host}{api_path}");

    let qb = QueryBuilder::new().add_param("token", &cfg.get_token()).add_param("zone", &zone);

    let body = match app_config.tdns_client.get_body(&url, &Some(qb)).await {
        Ok(b) => b,
        Err(error) => {
            return Err(errors::make_http_error(command_name, host, error));
        }
    };

    let resp = match serde_json::from_str::<Response>(&body) {
        Ok(r) => r,
        Err(error) => {
            return Err(errors::make_json_error(command_name, error));
        }
    };

    println!("Response status: {}", resp.status);

    Ok(())
}
