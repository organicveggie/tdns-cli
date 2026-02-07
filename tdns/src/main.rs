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
        default_value = "~/.config/tdns-cli/config.json",
        help = "Full path to config file"
    )]
    config_file: String,

    #[command(subcommand)]
    command: tdns::Command,
}

#[tokio::main]
async fn main() {
    let config_manager = tdns::config::ConfigFileManager;
    let http_client = match tdns::client::TdnsHttpClient::new() {
        Ok(c) => c,
        Err(error) => panic!("Error creating HTTP client: {}", error),
    };

    let cli = Cli::parse();
    tdns::run_cli(config_manager, http_client, &cli.config_file, &cli.command).await;
}
