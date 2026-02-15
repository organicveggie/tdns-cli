use clap::Subcommand;

use crate::config;
use crate::errors::{self, TdnsError};
use crate::tables::TableStyles;

pub mod add;
pub mod enable;
pub mod enums;
pub mod helpers;

mod get_records;
mod records;

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Add a record to a zone")]
    Add {
        #[arg(help = "Domain name of the record to add")]
        domain: String,

        #[arg(long = "ttl", help = "TTL in seconds for the record to add")]
        ttl: Option<u32>,

        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Overwrite existing record if it exists"
        )]
        overwrite: bool,

        #[arg(long = "comments", help = "Comments for the record to add")]
        comments: Option<String>,

        #[arg(
            long = "expiry-ttl",
            help = "Expiry TTL in seconds for the record to add",
            long_help = "Set to automatically delete the record when the value in seconds elapses since 
the record’s last modified time."
        )]
        expiry_ttl: Option<u32>,

        #[command(subcommand)]
        add_command: add::RecordTypeCommand,
    },

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
    pub async fn run(
        &self,
        app_config: &config::ApplicationConfig,
        config_file_name: &str,
        zone: String,
    ) -> Result<(), errors::TdnsError> {
        match self {
            Command::Add { domain, ttl, overwrite, comments, expiry_ttl, add_command } => {
                return add_command
                    .run(
                        app_config,
                        config_file_name,
                        zone,
                        domain.clone(),
                        *overwrite,
                        comments.clone(),
                        ttl.clone(),
                        expiry_ttl.clone(),
                    )
                    .await;
            }

            Command::Disable => {
                return enable::run(app_config, config_file_name, zone, enable::Mode::Disable)
                    .await;
            }
            Command::Enable => {
                return enable::run(app_config, config_file_name, zone, enable::Mode::Enable).await;
            }
            Command::List { domain, detail, table_style } => {
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
                return cmd.execute().await;
            }
        }
    }
}
