use clap::{Parser, Subcommand};
use std::error::Error;

pub mod config;
pub mod errors;
pub mod zone;
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
    List {
        #[arg(
            short = 'o',
            long = "sort",
            help = "Sort zones in ascending alphabetically"
        )]
        sort_asc: Option<bool>,
    },
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
        Command::List { sort_asc } => {
            let sort_mode = zones::ZoneSortMode::from_option(sort_asc);
            let cmd = match zones::ListCmd::create(&cli.config_file, sort_mode) {
                Ok(cmd) => cmd,
                Err(error) => panic!("failed to list zones: {}", error),
            };
            match cmd.execute().await {
                Ok(()) => {}
                Err(error) => {
                    eprintln!("Error: {}", error);
                    let mut source: Option<&(dyn Error + 'static)> = error.source();
                    while let Some(cause) = source {
                        eprintln!("Caused by: {}", cause);
                        source = cause.source();
                    }
                }
            }
        }
        Command::Zone { zone, zone_command } => match zone_command {
            ZoneCommand::Disable => {
                println!("disable {:?}", zone);
            }
            ZoneCommand::Enable => {
                println!("enable {:?}", zone);
            }
            ZoneCommand::List { domain } => {
                if let Some(domain_name) = domain {
                    println!("list records for {:?} in {:?}", domain_name, zone);
                } else {
                    println!("list records for {:?}", zone);
                }
                let cmd = match zone::GetRecordsCmd::create(
                    &cli.config_file,
                    zone.clone(),
                    domain.clone(),
                ) {
                    Ok(cmd) => cmd,
                    Err(error) => panic!("failed to list records for {}: {}", zone, error),
                };
                match cmd.execute().await {
                    Ok(()) => {}
                    Err(error) => {
                        eprintln!("Error: {}", error);
                        let mut source: Option<&(dyn Error + 'static)> = error.source();
                        while let Some(cause) = source {
                            eprintln!("Caused by: {}", cause);
                            source = cause.source();
                        }
                    }
                }
            }
            ZoneCommand::Resync => {
                println!("resync {:?}", zone);
            }
        },
    }
}
