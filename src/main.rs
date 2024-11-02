use std::path::PathBuf;

use package_manager::ChangelogQuery;
use clap::{Parser, Subcommand};
use storage::{Config, Data, TomlStorage};

mod package_manager;
mod storage;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    StorageError(storage::Error),
    PackageManagerError(package_manager::Error)
}

impl From<storage::Error> for Error {
    fn from(value: storage::Error) -> Self {
        Error::StorageError(value)
    }
}

impl From<package_manager::Error> for Error {
    fn from(value: package_manager::Error) -> Self {
        Error::PackageManagerError(value)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::StorageError(err) => Some(err),
            Error::PackageManagerError(err) => Some(err)
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::StorageError(err) => err.fmt(f),
            Error::PackageManagerError(err) => err.fmt(f)
        }
    }
}

fn handle_storage_result<T>(config_result: std::result::Result<T, storage::Error>) -> Result<Option<T>> {
    let result = match config_result {
        Err(storage::Error::FileAlreadyExists) => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(e) => Err(e)
    };

    Ok(result?)
}

fn get_changelogs(query: Option<String>) -> Result<String> {
    let config = Config::fetch()?;
    let pkg_manager = package_manager::get_package_manager(&config.package)?;
    let ref changelog_query = ChangelogQuery { name: query };

    Ok(pkg_manager.get_cached_changelogs(changelog_query)?)
}

fn perform_test() -> Result<()> {
    let config = Config::fetch()?;
    let pkg_manager = package_manager::get_package_manager(&config.package)?;
    let ref changelog_query = ChangelogQuery { name: None };

    let updates = pkg_manager.check_update()?;
    if updates.is_empty() {
        println!("No updates available.")
    } else {
        println!("Available updates:");
        for update in updates {
            println!("{} ({}) -> ({})", update.name, update.old_version, update.new_version)
        }
    }

    pkg_manager.download_update()?;
    let changelogs = pkg_manager.get_cached_changelogs(changelog_query)?;
    println!("Changelog:\n{}", changelogs);

    pkg_manager.do_update()?;

    Ok(())
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
    },
    Changelog {
        #[arg(long = "query", short = 'q')]
        query: Option<String>,
    },
    Test
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Init { config: path_opt } => {
            let path_opt = handle_storage_result(Config::init(path_opt))?;
            handle_storage_result(Data::init(None))?;

            if let Some(s) = path_opt.as_ref().and_then(|path| path.to_str()) {
                println!("Wrote configuration to {}", s)
            }
        }
        Command::Changelog { query } => {
            let changelogs = get_changelogs(query)?;
            println!("{}", changelogs);
        },
        Command::Test => {
            let test_result = perform_test();
            match test_result {
                Ok(_) => println!("Test succeeded."),
                Err(error) => eprintln!("Error: {}", error)
            }
        }
    }

    Ok(())
}
