use std::fs;
use std::path::Path;

use crate::storage::PackageConfig;

use super::{utilities, Error};
use super::error::Result;

pub struct ChangelogQuery {
    pub name: Option<String>
}

pub struct PackageChangelogResult {
    pub name: String,
    pub changelogs: Vec<String>
}

pub struct PackageUpdateItem {
    pub name: String,
    pub old_version: Option<String>,
    pub new_version: Option<String>
}

impl std::fmt::Display for PackageUpdateItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;

        if let Some(ref new_version) = self.new_version {
            write!(f, " ({})", new_version)?;

            if let Some(ref old_version) = self.old_version {
                write!(f, " -> ({})", old_version)?;
            }
        }

        Ok(())
    }
}

pub trait PackageManager {
    fn get_cached_changelogs(&self, query: &ChangelogQuery) -> Result<String> {
        if let Some(ref path) = self.get_config().cached_package_path {
            self.get_dir_changelogs(query, path)
        } else {
            Err(Error::UnkownCachedPackagePath)
        }
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
            for (i, changelog) in changelogs.iter().enumerate() {
                if i > 0 {
                    changelog_string.push_str("\n\n");
                }
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

    fn get_config(&self) -> &PackageConfig;

    /// Uses package manager specific logic to open the package file at the given path, and returns the package name
    /// along with a list of changelog entries.
    fn get_package_changelogs_result(&self, query: &ChangelogQuery, path: &Path) -> Result<PackageChangelogResult>;

    fn check_update(&self) -> Result<Vec<PackageUpdateItem>>;

    fn download_update(&self) -> Result<()> {
        let config = self.get_config();
        utilities::run_shell_command(config.download_command.as_str(), true, |err| Error::DownloadError(err))
    }

    fn do_update(&self, interactive: bool) -> Result<()> {
        let config = self.get_config();
        if interactive {
            utilities::run_interactive_shell_command(config.update_command.as_str(), true)
        } else {
            utilities::run_shell_command(config.noconfirm_update_command.as_str(), true,  |err| Error::UpdateError(err))
        }
    }
}