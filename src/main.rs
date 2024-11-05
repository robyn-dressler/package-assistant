use std::path::PathBuf;

use package::ChangelogQuery;
use clap::{Parser, Subcommand};
use storage::{Config, Data, TomlStorage};

mod package;
mod storage;
mod gui;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    StorageError(storage::Error),
    PackageManagerError(package::Error)
}

impl From<storage::Error> for Error {
    fn from(value: storage::Error) -> Self {
        Error::StorageError(value)
    }
}

impl From<package::Error> for Error {
    fn from(value: package::Error) -> Self {
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

#[derive(Debug, Parser)]
#[command(name = "package-assistant")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Initializes configuration and systemd services")]
    Init {
        #[arg(long = "config", short = 'c', help = "Copies the configuration from the provided file")]
        config: Option<PathBuf>,
    },
    #[command(about = "Uses the system's package manager to check whether there are update available.")]
    CheckUpdate {
        #[arg(long = "download", short = 'd', help = "If there are pending updates, downloads and caches packages locally.")]
        download: bool
    },
    #[command(about = "Uses the system's package manager to run an update.")]
    Update {
        #[arg(long = "gui", help = "Starts the update in a new GUI window.")]
        gui: bool,
        #[arg(long = "noconfirm", short = 'y', help = "Runs the update in a non-interactive mode, and attempts to solve conflicts automatically.")]
        no_confirm: bool
    },
    #[command(about = "Lists the changelogs for any cached packages")]
    Changelog {
        #[arg(long = "query", short = 'q', help = "Filters changelogs by package name")]
        query: Option<String>,
    },
    Gui,
    #[cfg(debug_assertions)]
    #[command(about = "Verifies that package-assistant runs properly")]
    Test
}

fn main() {
    let args = Cli::parse();
    let result = match args.command {
        Command::Init { config: path_opt } => init(path_opt),
        Command::CheckUpdate { download } => check_update(download),
        Command::Update { gui, no_confirm } => update(gui, no_confirm),
        Command::Changelog { query } => changelog(query),
        Command::Gui => gui(),
        #[cfg(debug_assertions)]
        Command::Test => perform_test(),
    };

    match result {
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        },
        _ => std::process::exit(0)
    }
}

fn init(path_opt: Option<PathBuf>) -> Result<()> {
    let output_path_opt = handle_storage_result(Config::init(path_opt))?;
    handle_storage_result(Data::init(None))?;

    if let Some(s) = output_path_opt.as_ref().and_then(|path| path.to_str()) {
        println!("Wrote configuration to {}", s)
    }

    Ok(())
}

fn check_update(download: bool) -> Result<()> {
    let config = Config::fetch()?;
    let pkg_manager = package::get_package_manager(&config.package)?;
    let updates = pkg_manager.check_update()?;

    if updates.is_empty() {
        println!("No updates available.");
        return Ok(())
    } else {
        println!("Available updates:");
        for update in updates {
            println!("{}", update);
        }
    }

    if download || config.service.download_in_background {
        pkg_manager.download_update()?;
        println!("Updates downloaded.");
    }

    Ok(())
}

fn update(gui: bool, no_confirm: bool) -> Result<()> {
    let config = Config::fetch()?;
    let pkg_manager = package::get_package_manager(&config.package)?;

    pkg_manager.do_update(!no_confirm)?;

    Ok(())
}

fn changelog(query: Option<String>) -> Result<()> {
    let config = Config::fetch()?;
    let pkg_manager = package::get_package_manager(&config.package)?;
    let ref changelog_query = ChangelogQuery { name: query };
    let changelogs = pkg_manager.get_cached_changelogs(changelog_query)?;
    println!("{}", changelogs);
    Ok(())
}

fn gui() -> Result<()> {
    gui::start_app();
    Ok(())
}

#[cfg(debug_assertions)]
fn perform_test() -> Result<()> {
    let config = Config::fetch()?;
    let pkg_manager = package::get_package_manager(&config.package)?;
    let ref changelog_query = ChangelogQuery { name: None };

    let updates = pkg_manager.check_update()?;
    if updates.is_empty() {
        println!("No updates available.");
    } else {
        println!("Available updates:");
        for update in updates {
            println!("{}", update);
        }
    }

    pkg_manager.download_update()?;
    let changelogs = pkg_manager.get_cached_changelogs(changelog_query)?;
    println!("Changelog:\n{}", changelogs);

    pkg_manager.do_update(false)?;

    println!("Test succeeded!");
    Ok(())
}

fn handle_storage_result<T>(config_result: std::result::Result<T, storage::Error>) -> Result<Option<T>> {
    let result = match config_result {
        Err(storage::Error::FileAlreadyExists) => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(e) => Err(e)
    };

    Ok(result?)
}