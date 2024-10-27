use std::path::PathBuf;

use changelog::ChangelogQuery;
use clap::{Parser, Subcommand};
use storage::{Config, Data, TomlStorage};

mod storage;
mod changelog;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    StorageError(storage::Error),
    ChangelogError(changelog::Error),
    PackagePathUndefined,
}

impl From<storage::Error> for Error {
    fn from(value: storage::Error) -> Self {
        Error::StorageError(value)
    }
}

impl From<changelog::Error> for Error {
    fn from(value: changelog::Error) -> Self {
        Error::ChangelogError(value)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::StorageError(err) => Some(err),
            Error::ChangelogError(err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::StorageError(err) => err.fmt(f),
            Error::ChangelogError(err) => err.fmt(f),
            Error::PackagePathUndefined => write!(f, "must configure 'cached_package_path' before calling changelog command")
        }
    }
}

fn get_changelogs(query: Option<String>) -> Result<String> {
    let config_result = Config::fetch()?;
    let data_result = Data::fetch()?;

    if let Some(path) = config_result.service.cached_package_path {
        let query = ChangelogQuery { name: query, timestamp: data_result.update_timestamp };
        Ok(changelog::get_dir_changelogs(&query, path)?)
    } else {
        Err(Error::PackagePathUndefined)
    }
}

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
            let config_result = Config::init(path_opt);
            let data_result = Data::init(None);
            match (config_result, data_result) {
                (Err(config_error), _) => {
                    println!("Error with configuration: {}", config_error);
                },
                (_, Err(data_error)) => {
                    println!("Error initializing data file: {}", data_error);
                },
                (Ok(config_dir), _) => {
                    if let Some(s) = config_dir.to_str() {
                        println!("Wrote configuration to {}", s)
                    }
                }
            };
        }
        Command::Changelog {
            query,
        } => {
            match get_changelogs(query) {
                Ok(changelogs) => println!("{}", changelogs),
                Err(err) => println!("Error: {}", err)
            }
        }
        Command::CheckUpdates { download } => {
            println!("Checking for updates!");
            if download {
                println!("Downloading available packages...");
            }
        }
    }
}
