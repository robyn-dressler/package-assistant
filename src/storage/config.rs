use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::toml::TomlStorage;

const CONFIG_HOME: &str = "XDG_CONFIG_HOME";
const DEFAULT_CONFIG_PATH: &str = ".config";
const CONFIG_FILE_NAME: &str = "settings.toml";

/* Config structs */

#[derive(Deserialize, Serialize)]
pub struct Config {
    service: ServiceConfig
}

impl TomlStorage for Config {
    fn default_directory() -> &'static str {
        DEFAULT_CONFIG_PATH
    }

    fn directory_env_var() -> &'static str {
        CONFIG_HOME
    }

    fn file_name() -> &'static str {
        CONFIG_FILE_NAME
    }
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