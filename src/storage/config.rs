use serde::{de::Error, Deserialize, Serialize};
use std::path::PathBuf;

use super::toml::TomlStorage;

const CONFIG_PATH: &str = "/etc";
const CONFIG_FILE_NAME: &str = "settings.toml";

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub service: ServiceConfig,
    pub package: PackageConfig
}

impl TomlStorage for Config {
    fn default_directory() -> &'static str {
        CONFIG_PATH
    }

    fn file_name() -> &'static str {
        CONFIG_FILE_NAME
    }
}

#[derive(Deserialize, Serialize)]
pub struct ServiceConfig {
    pub enable_service: bool,
    pub update_check_frequency: u32,
    pub download_in_background: bool,
    pub update_on_reboot: bool,
}

#[derive(Deserialize, Serialize)]
pub struct PackageConfig {
    pub package_manager: Option<PackageManagerType>,
    pub download_command: String,
    pub update_command: String,
    pub noconfirm_update_command: String,
    pub cached_package_path: Option<PathBuf>
}

pub enum PackageManagerType {
    Zypper,
    Dnf,
    Apt,
    Pacman
}

impl Serialize for PackageManagerType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        match self {
            PackageManagerType::Zypper => serializer.serialize_str("zypper"),
            PackageManagerType::Dnf => serializer.serialize_str("dnf"),
            PackageManagerType::Apt => serializer.serialize_str("apt"),
            PackageManagerType::Pacman => serializer.serialize_str("pacman")
        }
    }
}

impl<'de> Deserialize<'de> for PackageManagerType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "zypper" => Ok(PackageManagerType::Zypper),
            "dnf" => Ok(PackageManagerType::Dnf),
            "apt" => Ok(PackageManagerType::Apt),
            "pacman" => Ok(PackageManagerType::Pacman),
            _ => Err(Error::custom("'package_manager' must be set to either \"zypper\", \"dnf\", \"apt\", or \"pacman\" in settings"))
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service: ServiceConfig {
                enable_service: true,
                update_check_frequency: 30,
                download_in_background: true,
                update_on_reboot: true
            },
            package: PackageConfig {
                package_manager: None,
                download_command: String::from(""),
                update_command: String::from(""),
                noconfirm_update_command: String::from(""),
                cached_package_path: None
            }
        }
    }
}