use std::process::Command;
use std::{fs, io};
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    RPMError(rpm::Error),
    Utf8StringError(std::string::FromUtf8Error),
    ParseIntError(std::num::ParseIntError),
    NoChangelogsForPackage,
    NoChangelogsInDirectory,
    PackageNameDoesNotMatch(String, String),
    InvalidRPMResponse,
    RPMCommandError(String)
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

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IO(err) => Some(err),
            Error::RPMError(err) => Some(err),
            Error::Utf8StringError(err) => Some(err),
            Error::ParseIntError(err) => Some(err),
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
            Error::NoChangelogsForPackage => write!(f, "package has no changelogs to display"),
            Error::NoChangelogsInDirectory => write!(f, "could not find any packages containing changelogs"),
            Error::PackageNameDoesNotMatch(name, query) => write!(f, "package '{}' does not match the query '{}'", name, query),
            Error::RPMCommandError(error_string) => write!(f, "rpm command failed: {}", error_string),
            Error::InvalidRPMResponse => write!(f, "rpm query returned an unexpected response")
        }
    }
}

pub struct ChangelogQuery {
    pub name: Option<String>
}

/// Within the given `path`, for all package names that match the `query`, recursively finds all changelogs
/// for each package, and appends them to a single output string.
pub fn get_dir_changelogs(query: &ChangelogQuery, path: PathBuf) -> Result<String> {
    let paths = fs::read_dir(path)?;
    let changelogs = paths.map(|item| {
        let entry = item?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            get_dir_changelogs(&query, entry.path())
        } else {
            get_rpm_changelogs(&query, entry.path())
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
fn get_rpm_changelogs(query: &ChangelogQuery, path: PathBuf) -> Result<String> {
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

fn matches_query(name: &str, query: &str) -> bool {
    name.starts_with(query)
}

/// Calls rpm command to get the latest changelog timestamp of an installed package
fn get_installed_pkg_timestamp(name: &str) -> Result<u64> {
    let output = Command::new("rpm")
        .args(["-q", name, "--qf", "%{CHANGELOGTIME}"])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    if !stderr.is_empty() {
        return Err(Error::RPMCommandError(stderr))
    }

    if let Some(first_line) = stdout.lines().next() {
        Ok(first_line.parse::<u64>()?)
    } else {
        Err(Error::InvalidRPMResponse)
    }
}