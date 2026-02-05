use clap::Subcommand;
use std::error::Error;

use crate::errors::{self, TdnsError};
use crate::tables::TableStyles;

pub mod helpers;

mod get_records;
mod records;

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Disables an authoritative zone")]
    Disable,

    #[command(about = "Enables an authoritative zone")]
    Enable,

    #[command(about = "List records in a zone")]
    List {
        #[arg(help = "Optional domain name to list. Skip to list all records.")]
        domain: Option<String>,

        #[arg(
            value_enum,
            short = 'd',
            long = "detail",
            help = "Level of detail to include when printing zone records",
            default_value_t = get_records::ZoneRecordDetailLevel::Summary,
        )]
        detail: get_records::ZoneRecordDetailLevel,

        #[arg(
            value_enum,
            long = "table_style",
            help = "Table style to use when printing zone records",
            default_value_t = TableStyles::Ascii,
        )]
        table_style: TableStyles,
    },
}

impl Command {
    pub async fn run(&self, config_file_name: &str, zone: String) -> Result<(), errors::TdnsError> {
        match self {
            Command::Disable => {
                println!("disable {:?}", zone);
            }
            Command::Enable => {
                println!("enable {:?}", zone);
            }
            Command::List {
                domain,
                detail,
                table_style,
            } => {
                if let Some(domain_name) = domain {
                    println!("list records for {:?} in {:?}", domain_name, zone);
                } else {
                    println!("list records for {:?}", zone);
                }
                let cmd = match get_records::GetRecordsCmd::create(
                    config_file_name,
                    zone.clone(),
                    domain.clone(),
                    detail.clone(),
                    table_style.clone(),
                ) {
                    Ok(cmd) => cmd,
                    Err(error) => {
                        return Err(TdnsError::ConfigFileError {
                            command: get_records::CMD_NAME.to_string(),
                            source: error,
                        });
                    }
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
        }
        return Ok(());
    }
}
