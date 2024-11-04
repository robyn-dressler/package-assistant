use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    RPMError(rpm::Error),
    Utf8StringError(std::string::FromUtf8Error),
    ParseIntError(std::num::ParseIntError),
    XMLError(quick_xml::errors::Error),
    XMLAttributeError(quick_xml::events::attributes::AttrError),
    RegexError(regex::Error),
    NoChangelogsForPackage,
    NoChangelogsInDirectory,
    PackageNameDoesNotMatch(String, String),
    InvalidRPMResponse,
    RPMCommandError(String),
    UnsupportedPackageManager,
    UnkownCachedPackagePath,
    EmptyCommand,
    DownloadError(String),
    UpdateError(String),
    ZypperError(String),
    DnfError(String)
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

impl From<regex::Error> for Error {
    fn from(value: regex::Error) -> Self {
        Error::RegexError(value)
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
            Error::RegexError(err) => Some(err),
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
            Error::RegexError(err) => err.fmt(f),
            Error::NoChangelogsForPackage => write!(f, "package has no changelogs to display"),
            Error::NoChangelogsInDirectory => write!(f, "could not find any packages containing changelogs"),
            Error::PackageNameDoesNotMatch(name, query) => write!(f, "package '{}' does not match the query '{}'", name, query),
            Error::RPMCommandError(error_string) => write!(f, "rpm command failed: {}", error_string),
            Error::InvalidRPMResponse => write!(f, "rpm query returned an unexpected response"),
            Error::UnsupportedPackageManager => write!(f, "'package_manager' in settings is either empty or not supported"),
            Error::UnkownCachedPackagePath => write!(f, "'cached_package_path' must be provided in settings"),
            Error::EmptyCommand => write!(f, "update and download commands must be provided in settings"),
            Error::DownloadError(error_string) => write!(f, "failed to download packages: {}", error_string),
            Error::UpdateError(error_string) => write!(f, "failed to run update: {}", error_string),
            Error::ZypperError(error_string) => write!(f, "zypper command failed: {}", error_string),
            Error::DnfError(error_string) => write!(f, "dnf command failed: {}", error_string),
        }
    }
}