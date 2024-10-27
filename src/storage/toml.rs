use std::{fs, path::PathBuf};

use serde::{de::DeserializeOwned, Serialize};

use super::error::Error;

const USER_HOME: &str = "HOME";
const PROGRAM_NAME: &str = "package-assistant";

type Result<T> = std::result::Result<T, Error>;

pub trait TomlStorage: Default + DeserializeOwned + Serialize {
    fn new() -> Self {
        Default::default()
    }

    fn from_toml_str(contents: &str) -> Result<Self> {
        let data = toml::from_str::<Self>(contents)?;
        Ok(data)
    }

    fn to_toml_str(&self) -> Result<String> {
        let serialized_value = toml::to_string(&self)?;
        Ok(serialized_value)
    }

    /// Gets the saved TOML file as a struct
    fn fetch() -> Result<Self> {
        let path = Self::get_file_path()?;
        let contents = fs::read_to_string(path)?;
        let data = Self::from_toml_str(contents.as_str())?;

        Ok(data)
    }

    /// Saves the provided struct to the filesystem as TOML
    fn save(data: Self) -> Result<()> {
        let path = Self::get_file_path()?;
        let contents = data.to_toml_str()?;
        fs::write(&path, contents)?;

        Ok(())
    }

    /// Finds the standard directory as described in the XDG specification. Returns
    /// `Error::DirUndefined` if it is unable to resolve the directory using the existing
    /// environment variables.
    fn get_dir_path() -> Result<PathBuf> {
        let home_dir = std::env::var_os(USER_HOME);
        let data_home = std::env::var_os(Self::directory_env_var());

        match data_home {
            Some(c) if !c.is_empty() => Ok(PathBuf::from(c)),
            _ => {
                if let Some(home) = home_dir {
                    let mut result = PathBuf::from(home);
                    result.push(Self::default_directory());
                    result.push(PROGRAM_NAME);

                    Ok(result)
                } else {
                    Err(Error::DirUndefined)
                }
            }
        }
    }

    /// Gets the path that the file will be saved to
    fn get_file_path() -> Result<PathBuf> {
        let mut path = Self::get_dir_path()?;
        path.push(Self::file_name());
        Ok(path)
    }

    /// Creates a toml file if it doesn't already exist. If `custom_path` is provided,
    /// it deletes any existing toml file and copies the provided file
    /// to the predefined directory. Returns the path to the saved file.
    fn init(custom_path: Option<PathBuf>) -> Result<PathBuf>{
        // Retrieve directory path and create it if it doesn't exist
        let data_dir = Self::get_dir_path()?;
        fs::create_dir_all(&data_dir)?;

        // Append file name to path
        let file_path = Self::get_file_path()?;

        // Copy from provided file if it exists
        if let Some(path) = custom_path {
            let contents = fs::read_to_string(path)?;
            let data = Self::from_toml_str(contents.as_str())?;
            Self::save(data)?;

        //Create a fresh data file with the default settings
        } if fs::exists(&file_path)? {
            return Err(Error::FileAlreadyExists)
        } else {
            let data = Self::new();
            Self::save(data)?;
        }

        Ok(file_path)
    }

    fn file_name() -> &'static str;

    /// The environment variable to use to retrieve the parent directory of the file,
    /// e.g. XDG_CONFIG_HOME
    fn directory_env_var() -> &'static str;

    /// The fallback directory to save the file to relative to the user's home directory, e.g. .config
    fn default_directory() -> &'static str;
}