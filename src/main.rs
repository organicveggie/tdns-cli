use std::rc::Rc;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "tdns-cli")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(
        short,
        long,
        global = true,
        default_value = "~/.config/tdns-cli/config.toml",
        help = "Full path to config file"
    )]
    config_file: String,

    #[arg(
        long,
        global = true,
        default_value_t = false,
        help = "Allow invalid TLS certificates when connecting to the TDNS server"
    )]
    allow_invalid_certificates: bool,

    #[command(subcommand)]
    command: tdns::Command,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config_manager = tdns::config::ConfigFileManager;
    let http_client = match tdns::client::TdnsHttpClient::new(cli.allow_invalid_certificates) {
        Ok(c) => c,
        Err(error) => panic!("Error creating HTTP client: {}", error),
    };

    let app_config = tdns::config::ApplicationConfig {
        config_manager: Box::new(config_manager),
        tdns_client: Rc::new(http_client),
        output: tdns::config::OutputTarget::stdout(),
    };

    tdns::run_cli(&app_config, &cli.config_file, &cli.command).await;
}
