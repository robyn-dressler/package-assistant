use std::process::{Command, Output};
use std::{fs, io};
use std::path::Path;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::storage::{PackageConfig, PackageManagerType};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    RPMError(rpm::Error),
    Utf8StringError(std::string::FromUtf8Error),
    ParseIntError(std::num::ParseIntError),
    XMLError(quick_xml::errors::Error),
    XMLAttributeError(quick_xml::events::attributes::AttrError),
    NoChangelogsForPackage,
    NoChangelogsInDirectory,
    PackageNameDoesNotMatch(String, String),
    InvalidRPMResponse,
    RPMCommandError(String),
    UnsupportedPackageManager,
    UnkownCachedPackagePath,
    DownloadError(String),
    UpdateError(String),
    ZypperError(String)
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IO(value)
    }
}

impl From<rpm::Error> for Error {
    fn from(value: rpm::Error) -> Self {
        Error::RPMError(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Error::Utf8StringError(value)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Error::ParseIntError(value)
    }
}

impl From<quick_xml::errors::Error> for Error {
    fn from(value: quick_xml::errors::Error) -> Self {
        Error::XMLError(value)
    }
}

impl From<quick_xml::events::attributes::AttrError> for Error {
    fn from(value: quick_xml::events::attributes::AttrError) -> Self {
        Error::XMLAttributeError(value)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IO(err) => Some(err),
            Error::RPMError(err) => Some(err),
            Error::Utf8StringError(err) => Some(err),
            Error::ParseIntError(err) => Some(err),
            Error::XMLError(err) => Some(err),
            Error::XMLAttributeError(err) => Some(err),
            _ => None
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(err) => err.fmt(f),
            Error::RPMError(err) => err.fmt(f),
            Error::Utf8StringError(err) => err.fmt(f),
            Error::ParseIntError(err) => err.fmt(f),
            Error::XMLError(err) => err.fmt(f),
            Error::XMLAttributeError(err) => err.fmt(f),
            Error::NoChangelogsForPackage => write!(f, "package has no changelogs to display"),
            Error::NoChangelogsInDirectory => write!(f, "could not find any packages containing changelogs"),
            Error::PackageNameDoesNotMatch(name, query) => write!(f, "package '{}' does not match the query '{}'", name, query),
            Error::RPMCommandError(error_string) => write!(f, "rpm command failed: {}", error_string),
            Error::InvalidRPMResponse => write!(f, "rpm query returned an unexpected response"),
            Error::UnsupportedPackageManager => write!(f, "'package_manager' must be set to either \"zypper\", \"dnf\", \"apt\", or \"pacman\" in settings"),
            Error::UnkownCachedPackagePath => write!(f, "'cached_package_path' must be provided in settings"),
            Error::DownloadError(error_string) => write!(f, "failed to download packages: {}", error_string),
            Error::UpdateError(error_string) => write!(f, "failed to run update: {}", error_string),
            Error::ZypperError(error_string) => write!(f, "zypper command failed: {}", error_string),
        }
    }
}

pub struct ChangelogQuery {
    pub name: Option<String>
}

pub struct PackageChangelogResult {
    name: String,
    changelogs: Vec<String>
}

pub struct PackageUpdateItem {
    pub name: String,
    pub old_version: String,
    pub new_version: String
}

pub fn get_package_manager<'a>(config: &'a PackageConfig) -> Result<Box<dyn PackageManager + 'a>> {
    match config.package_manager {
        Some(PackageManagerType::Zypper) => Ok(Box::new(ZypperManager { config })),
        _ => Err(Error::UnsupportedPackageManager)
    }
}

fn run_shell_command<F>(command: &str, get_error: F) -> Result<()>
where F: Fn(String) -> Error {
    let output = Command::new("sh")
        .args(["-c", command])
        .output()?;

    process_cmd_output(output, get_error)?;

    Ok(())
}

fn process_cmd_output<F>(output: Output, get_error: F) -> Result<String>
where F: Fn(String) -> Error {
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(get_error(stderr))
    } else {
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    }
}

pub trait PackageManager {
    fn get_cached_changelogs(&self, query: &ChangelogQuery) -> Result<String> {
        let path = self.get_cached_package_path()?;
        self.get_dir_changelogs(query, path)
    }

    /// Within the given `path`, for all package names that match the `query`, recursively finds all changelogs
    /// for each package, and appends them to a single output string.
    fn get_dir_changelogs(&self, query: &ChangelogQuery, path: &Path) -> Result<String> {
        let subpaths = fs::read_dir(path)?;
        let changelogs = subpaths.map(|item| {
            let entry = item?;
            let file_type = entry.file_type()?;
    
            if file_type.is_dir() {
                self.get_dir_changelogs(&query, entry.path().as_path())
            } else {
                self.get_package_changelogs_string(&query, entry.path().as_path())
            }
        })
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap())
        .collect::<Vec<String>>();
    
        if changelogs.is_empty() {
            Err(Error::NoChangelogsInDirectory)
        } else {
            let mut changelog_string = String::new();
            for changelog in changelogs {
                changelog_string.push_str("\n\n");
                changelog_string.push_str(&changelog);
            }
        
            Ok(changelog_string)
        }
    }

    /// Gets all changelogs for a package at the given path, filtering out any changelogs that
    /// have a timestamp before the latest changelog of the corresponding installed package.
    /// If query does not match the package name, then returns `Error::PackageNameDoesNotMatch`.
    fn get_package_changelogs_string(&self, query: &ChangelogQuery, path: &Path) -> Result<String> {
        let PackageChangelogResult { name, changelogs } = self.get_package_changelogs_result(query, path)?;
    
        if changelogs.is_empty() {
            Err(Error::NoChangelogsForPackage)
        } else {
            let mut changelog_string = format!("==== {} ====", name);
            for changelog in changelogs {
                changelog_string.push_str("\n");
                changelog_string.push_str(&changelog);
            }
        
            Ok(changelog_string)
        }
    }

    fn get_cached_package_path(&self) -> Result<&Path> {
        if let Some(ref path) = self.get_config().cached_package_path {
            Ok(path)
        } else {
            Err(Error::UnkownCachedPackagePath)
        }
    }

    fn get_config(&self) -> &PackageConfig;

    /// Uses package manager specific logic to open the package file at the given path, and returns the package name
    /// along with a list of changelog entries.
    fn get_package_changelogs_result(&self, query: &ChangelogQuery, path: &Path) -> Result<PackageChangelogResult>;

    fn check_update(&self) -> Result<Vec<PackageUpdateItem>>;

    fn download_update(&self) -> Result<()> {
        let config = self.get_config();
        run_shell_command(config.download_command.as_str(), |err| Error::DownloadError(err))
    }

    fn do_update(&self) -> Result<()> {
        let config = self.get_config();
        run_shell_command(config.update_command.as_str(), |err| Error::UpdateError(err))
    }
}

/* Utility functions */
fn matches_query(name: &str, query: &str) -> bool {
    name.starts_with(query)
}

fn attr_to_string(attr: Attribute) -> String {
    String::from_utf8_lossy(attr.value.as_ref()).to_string()
}

/* RPM functions */
fn get_rpm_changelogs_result(query: &ChangelogQuery, path: &Path) -> Result<PackageChangelogResult> {
    let package = rpm::Package::open(path)?;
    let name = package.metadata.get_name()?;

    if let Some(ref query_name) = query.name {
        if !matches_query(name, query_name) {
            return Err(Error::PackageNameDoesNotMatch(name.to_owned(), (*query_name).clone()))
        }
    }

    let timestamp = get_installed_pkg_timestamp(name).unwrap_or(0);
    let changelogs = package.metadata.get_changelog_entries()?
        .into_iter()
        .filter(|c| c.timestamp > timestamp)
        .map(|c| c.description)
        .collect::<Vec<String>>();

    Ok(PackageChangelogResult { name: String::from(name), changelogs })
}

fn get_installed_pkg_timestamp(name: &str) -> Result<u64> {
    let output = Command::new("rpm")
        .args(["-q", name, "--qf", "%{CHANGELOGTIME}"])
        .output()?;

    let stdout = process_cmd_output(output, |err| Error::RPMCommandError(err))?;
    if let Some(first_line) = stdout.lines().next() {
        Ok(first_line.parse::<u64>()?)
    } else {
        Err(Error::InvalidRPMResponse)
    }
}

pub struct ZypperManager<'a> {
    config: &'a PackageConfig
}

impl<'a> PackageManager for ZypperManager<'a> {
    fn get_package_changelogs_result(&self, query: &ChangelogQuery, path: &Path) -> Result<PackageChangelogResult> {
        get_rpm_changelogs_result(query, path)
    }

    fn get_config(&self) -> &PackageConfig {
        self.config
    }
    
    fn check_update(&self) -> Result<Vec<PackageUpdateItem>> {
        let output = Command::new("zypper")
            .args(["--xmlout", "lu"])
            .output()?;
        let stdout = process_cmd_output(output, |err| Error::ZypperError(err))?;
        let mut reader = Reader::from_str(stdout.as_str());
        let mut items = Vec::new();

        loop {
            match reader.read_event()? {
                Event::Start(e) if e.name().as_ref() == b"update" =>{
                    let mut name: String = String::new();
                    let mut edition: String = String::new();
                    let mut edition_old: String = String::new();
                    let attributes = e.attributes();

                    for attr_result in attributes {
                        let attr = attr_result?;

                        match attr.key.as_ref() {
                            b"name" => name = attr_to_string(attr),
                            b"edition" => edition = attr_to_string(attr),
                            b"edition-old" => edition_old = attr_to_string(attr),
                            _ => ()
                        }
                    }

                    items.push(PackageUpdateItem { name, new_version: edition, old_version: edition_old });
                },
                Event::Eof => break,
                _ => ()
            }
        }

        Ok(items)
    }
}
