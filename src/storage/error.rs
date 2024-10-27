use std::io;

#[derive(Debug)]
pub enum Error {
    DirUndefined,
    FileAlreadyExists,
    TomlDeserializationError(toml::de::Error),
    TomlSerializationError(toml::ser::Error),
    IO(io::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IO(e) => Some(e),
            Error::TomlDeserializationError(e) => Some(e),
            Error::TomlSerializationError(e) => Some(e),
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
        Error::TomlDeserializationError(value)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(value: toml::ser::Error) -> Self {
        Error::TomlSerializationError(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DirUndefined => write!(f, "could not determine a directory to store data"),
            Error::FileAlreadyExists => write!(f, "file already exists"),
            Error::IO(err) => err.fmt(f),
            Error::TomlDeserializationError(err) => err.fmt(f),
            Error::TomlSerializationError(err) => err.fmt(f),
        }
    }
}