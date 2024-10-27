use std::path::PathBuf;

use clap::{Parser, Subcommand};
use storage::toml::TomlStorage;

mod storage;

#[derive(Debug, Parser)]
#[command(name = "package-assistant")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init {
        #[arg(long = "config", short = 'c')]
        config: Option<PathBuf>,
        #[arg(long = "service", short = 's')]
        service: bool,
    },
    Changelog {
        #[arg(long = "pending-only", short = 'p')]
        pending_only: bool,
        #[arg(long = "query", short = 'q')]
        query: Option<String>,
    },
    CheckUpdates {
        #[arg(long = "download", short = 'd')]
        download: bool,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Command::Init {
            config: path_opt,
            service,
        } => {
            let config_result = storage::config::Config::init(path_opt);
            match config_result {
                Err(config_error) => {
                    println!("Error with configuration: {}", config_error);
                }
                Ok(config_dir) => {
                    if let Some(s) = config_dir.to_str() {
                        println!("Wrote configuration to {}", s)
                    }
                }
            };
        }
        Command::Changelog {
            pending_only,
            query,
        } => {
            let pending_str = if pending_only { " pending " } else { " " };
            let query_str = if let Some(q) = query {
                format!("for query \"{}\"", q)
            } else {
                String::from("")
            };

            println!("Getting all{}changes {}", pending_str, query_str);
        }
        Command::CheckUpdates { download } => {
            println!("Checking for updates!");
            if download {
                println!("Downloading available packages...");
            }
        }
    }
}
