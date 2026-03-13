use clap::Subcommand;
use std::error::Error;

use crate::tables::TableStyles;

pub mod cli;
pub mod client;
pub mod config;
pub mod errors;
pub mod tables;
pub mod zone;
pub mod zones;

#[derive(Subcommand, Debug)]
pub enum Command {
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
            short,
            long = "sort",
            help = "Sort order. By default, zones are unsorted and returned in the order they are stored on the server.",
            default_value_t = cli::SortOrder::Unsorted,
        )]
        sort_order: cli::SortOrder,

        #[arg(
            short,
            long = "output",
            help = "Output format. By default, zones are printed in a table format.",
            default_value_t = cli::OutputFormat::Table,
        )]
        output_format: cli::OutputFormat,

        #[arg(
            value_enum,
            short,
            long,
            help = "Table style to use when printing zone records. Only applicable when the output format is table.",
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

pub async fn run_cli<'a, T>(app_config: &mut config::ApplicationConfig<'a, T>, config_file: &str, command: &Command) 
where T: std::io::Write {
    match command {
        Command::Init { force } => {
            println!("Init: force = {:?}", force);
            match app_config
                .config_manager
                .create_config_file(config_file, &force)
            {
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
            output_format,
            table_style,
        } => {
            let sort_mode = zones::ZoneSortMode::from_sort_order(&sort_order);
            let cmd = match zones::ListCmd::create(
                app_config,
                config_file,
                &output_format,
                sort_mode,
                table_style.clone(),
            ) {
                Ok(cmd) => cmd,
                Err(error) => panic!("failed to list zones: {}", error),
            };
            match cmd.execute(app_config.output).await {
                Ok(()) => {}
                Err(error) => {
                    print_tdns_error(&error);
                }
            }
        }
        Command::Zone { zone, zone_command } => {
            match zone_command
                .run(app_config, config_file, zone.clone())
                .await
            {
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
