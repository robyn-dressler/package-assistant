use serde::{Deserialize, Serialize};

use super::toml::TomlStorage;

const DATA_HOME: &str = "XDG_DATA_HOME";
const DEFAULT_DATA_PATH: &str = ".local/share";
const DATA_FILE_NAME: &str = "data.toml";

#[derive(Deserialize, Serialize)]
pub struct Data {
    pub update_timestamp: u64
}

impl TomlStorage for Data {
    fn default_directory() -> &'static str {
        DEFAULT_DATA_PATH
    }

    fn directory_env_var() -> &'static str {
        DATA_HOME
    }

    fn file_name() -> &'static str {
        DATA_FILE_NAME
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            update_timestamp: 0
        }
    }
}