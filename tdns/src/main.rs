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
    let cli = Cli::parse();
    tdns::run_cli(&cli.config_file, &cli.command).await;
}
