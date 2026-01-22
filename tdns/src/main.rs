use clap::{Parser, Subcommand};

pub mod config;
pub mod zones;

#[derive(Parser, Debug)]
#[command(name = "tdns-cli")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(
        short,
        long,
        global = true,
        default_value = "~/.config/tdns-cli/config.json",
        help = "Full path to config file"
    )]
    config_file: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Initialize config file")]
    Init {
        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Force overwriting existing config file"
        )]
        force: bool,
    },
    #[command(about = "List all zones")]
    List,
    #[command(about = "Perform actions on a specific zone")]
    Zone {
        #[arg(help = "Domain name of the zone")]
        zone: String,
        #[command(subcommand)]
        zone_command: ZoneCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ZoneCommand {
    #[command(about = "Disables an authoritative zone")]
    Disable,
    #[command(about = "Enables an authoritative zone")]
    Enable,
    #[command(about = "List records in a zone")]
    List {
        #[arg(help = "Optional domain name to list. Skip to list all records.")]
        domain: Option<String>,
    },
    #[command(about = "Resync a secondary or stub zone")]
    Resync,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Command::Init { force } => {
            println!("Init: force = {:?}", force);
            match config::create_config_file(&cli.config_file, force) {
                Ok(()) => {}
                Err(error) => {
                    println!("Error creating config file: {:?}", error);
                    return;
                }
            }
            println!("Created config file");
        }
        Command::List => {
            let cmd = match zones::ListCmd::create(&cli.config_file) {
                Ok(cmd) => cmd,
                Err(error) => panic!("failed to list zones: {}", error),
            };
            match cmd.execute().await {
                Ok(()) => {}
                Err(error) => panic!("failed to list zones: {}", error),
            }
        }
        Command::Zone { zone, zone_command } => {
            println!("zone subcommand: {:?}", zone);
            match zone_command {
                ZoneCommand::Disable => {
                    println!("disable {:?}", zone);
                }
                ZoneCommand::Enable => {
                    println!("enable {:?}", zone);
                }
                ZoneCommand::List { domain } => match domain {
                    Some(name) => {
                        println!("list records for {:?} in {:?}", name, zone);
                    }
                    _ => {
                        println!("list records for {:?}", zone);
                    }
                },
                ZoneCommand::Resync => {
                    println!("resync {:?}", zone);
                }
            }
        }
    }
}
