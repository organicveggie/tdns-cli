use clap::{Parser, Subcommand};
use std::error::Error;

use crate::tables::TableStyles;

pub mod cli;
pub mod client;
pub mod config;
pub mod errors;
pub mod tables;
pub mod zone;
pub mod zones;

use crate::config::ConfigManager;

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
            help = "Sort order. By default, zones are unsorted and returned in the order they are stored on the server.",
            default_value_t = cli::SortOrder::Unsorted,
        )]
        sort_order: cli::SortOrder,

        #[arg(
            value_enum,
            long = "table_style",
            help = "Table style to use when printing zone records",
            default_value_t = TableStyles::Ascii,
        )]
        table_style: TableStyles,
    },

    #[command(about = "Perform actions on a specific zone")]
    Zone {
        #[arg(help = "Domain name of the zone")]
        zone: String,
        #[command(subcommand)]
        zone_command: zone::Command,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Command::Init { force } => {
            println!("Init: force = {:?}", force);
            let cfg_mgr = config::ConfigFileManager;
            match cfg_mgr.create_config_file(&cli.config_file, force) {
                Ok(()) => {}
                Err(error) => {
                    println!("Error creating config file: {:?}", error);
                    return;
                }
            }
            println!("Created config file");
        }
        Command::List {
            sort_order,
            table_style,
        } => {
            let cfg_mgr = config::ConfigFileManager;
            let client = match client::TdnsHttpClient::new() {
                Ok(client) => client,
                Err(error) => panic!("Error creating HTTP client: {}", error),
            };
            let sort_mode = zones::ZoneSortMode::from_sort_order(sort_order);
            let cmd = match zones::ListCmd::create(
                &cfg_mgr,
                client,
                &cli.config_file,
                sort_mode,
                table_style.clone(),
            ) {
                Ok(cmd) => cmd,
                Err(error) => panic!("failed to list zones: {}", error),
            };
            match cmd.execute().await {
                Ok(()) => {}
                Err(error) => {
                    print_tdns_error(&error);
                }
            }
        }
        Command::Zone { zone, zone_command } => {
            match zone::Command::run(&zone_command, &cli.config_file, zone.clone()).await {
                Ok(()) => {}
                Err(error) => {
                    print_tdns_error(&error);
                }
            }
        }
    }
}

fn print_tdns_error(error: &errors::TdnsError) {
    eprintln!("Error: {}", error);
    let mut source: Option<&(dyn Error + 'static)> = error.source();
    while let Some(cause) = source {
        eprintln!("Caused by: {}", cause);
        source = cause.source();
    }
}
