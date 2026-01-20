use clap::{Parser, Subcommand};

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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Command::Init { force } => {
            println!("Init: force = {:?}", force);
        }
        Command::List => {
            println!("list zones");
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
