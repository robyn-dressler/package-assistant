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
    pub package_type: Option<PackageType>,
    pub check_update_command: String,
    pub download_command: String,
    pub update_command: String,
    pub cached_package_path: Option<PathBuf>
}

pub enum PackageType {
    RPM,
    Deb,
    Pkg
}

impl Serialize for PackageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        match self {
            PackageType::RPM => serializer.serialize_str("rpm"),
            PackageType::Deb => serializer.serialize_str("deb"),
            PackageType::Pkg => serializer.serialize_str("pkg")
        }
    }
}

impl<'de> Deserialize<'de> for PackageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "rpm" => Ok(PackageType::RPM),
            "deb" => Ok(PackageType::Deb),
            "pkg" => Ok(PackageType::Pkg),
            _ => Err(Error::custom("invalid package type"))
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
                package_type: None,
                check_update_command: String::from("pkcon get-updates"),
                download_command: String::from("pkcon update --only-download --background"),
                update_command: String::from("pkcon update"),
                cached_package_path: None
            }
        }
    }
}