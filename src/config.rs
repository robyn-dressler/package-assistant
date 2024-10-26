use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::io;
use std::path::PathBuf;

const USER_HOME: &str = "HOME";
const CONFIG_HOME: &str = "XDG_CONFIG_HOME";
const PROGRAM_NAME: &str = "package-assistant";
const DEFAULT_CONFIG_PATH: &str = ".config";
const CONFIG_FILE_NAME: &str = "settings.toml";

/* Error types */

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConfigDirNotFound,
    InvalidConfig(toml::de::Error),
    SerializationError(toml::ser::Error),
    IO(io::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IO(e) => Some(e),
            Error::InvalidConfig(e) => Some(e),
            Error::SerializationError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IO(value)
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::InvalidConfig(value)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(value: toml::ser::Error) -> Self {
        Error::SerializationError(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConfigDirNotFound => write!(f, "could not find config directory"),
            Error::IO(err) => err.fmt(f),
            Error::InvalidConfig(err) => {
                writeln!(f, "unable to read the provided config: ")?;
                err.fmt(f)
            }
            Error::SerializationError(err) => err.fmt(f),
        }
    }
}

/* Config structs */

#[derive(Deserialize, Serialize)]
pub struct Config {
    service: ServiceConfig
}

#[derive(Deserialize, Serialize)]
pub struct ServiceConfig {
    enable_service: bool,
    update_check_frequency: u32,
    download_in_background: bool,
    update_on_reboot: bool,
    check_update_command: String,
    download_command: String,
    update_command: String,
    update_on_reboot_command: String,
    cached_package_path: Option<PathBuf>
}

impl Config {
    fn new() -> Config {
        Config {
            ..Default::default()
        }
    }

    fn parse(contents: &str) -> Result<Config> {
        let config = toml::from_str::<Config>(contents)?;
        Ok(config)
    }

    fn serialize(&self) -> Result<String> {
        let serialized_value = toml::to_string(&self)?;
        Ok(serialized_value)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service: ServiceConfig {
                enable_service: true,
                update_check_frequency: 30,
                download_in_background: true,
                update_on_reboot: true,
                check_update_command: String::from("pkcon get-updates"),
                download_command: String::from("pkcon update --only-download --background"),
                update_command: String::from("pkcon update"),
                update_on_reboot_command: String::from("pkcon offline-trigger"),
                cached_package_path: None
            }
        }
    }
}

/* File IO */

/// Creates a configuration file if it doesn't already exist. If `custom_path` is provided,
/// it deletes any existing configuration file and copies the provided configuration file
/// to the config directory. Returns the path to the saved config file.
pub fn init(custom_path: Option<PathBuf>) -> Result<PathBuf> {
    // Retrieve config path and create it if it doesn't exist
    let config_dir = get_config_dir_path()?;
    fs::create_dir_all(&config_dir)?;

    // Append file name to path
    let config_file = build_config_file_path(config_dir)?;

    // Copy from provided config file if it exists
    if let Some(path) = custom_path {
        let contents = fs::read_to_string(path)?;
        let config = Config::parse(contents.as_str())?;
        save_config(config)?;

    //Create a fresh config file with the default settings
    } else if !fs::exists(&config_file)? {
        let config = Config::new();
        save_config(config)?;
    }

    Ok(config_file)
}

/// Gets the saved configuration as a `Config` struct
pub fn get_config() -> Result<Config> {
    let path = get_config_dir_path()?;
    let contents = fs::read_to_string(path)?;
    let config = Config::parse(contents.as_str())?;

    Ok(config)
}

/// Saves the provided config to the filesystem
pub fn save_config(config: Config) -> Result<()> {
    let path = get_config_file_path()?;
    let contents = config.serialize()?;
    fs::write(&path, contents)?;

    Ok(())
}

/// Finds the standard config directory as described in the XDG specification. Returns
/// `ConfigError::ConfigDirNotFound` if it is unable to resolve the directory using the
/// XDG_CONFIG_HOME or HOME environment variables.
pub fn get_config_dir_path() -> Result<PathBuf> {
    let home_dir = std::env::var_os(USER_HOME);
    let config_home = std::env::var_os(CONFIG_HOME);

    match config_home {
        Some(c) if !c.is_empty() => Ok(PathBuf::from(c)),
        _ => {
            if let Some(home) = home_dir {
                let mut result = PathBuf::from(home);
                result.push(DEFAULT_CONFIG_PATH);
                result.push(PROGRAM_NAME);

                Ok(result)
            } else {
                Err(Error::ConfigDirNotFound)
            }
        }
    }
}

fn build_config_file_path(config_dir: PathBuf) -> Result<PathBuf> {
    // Append file name to path
    let mut config_file = PathBuf::from(config_dir);
    config_file.push(CONFIG_FILE_NAME);

    Ok(config_file)
}

pub fn get_config_file_path() -> Result<PathBuf> {
    let config_dir = get_config_dir_path()?;
    build_config_file_path(config_dir)
}
