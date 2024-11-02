use std::path::Path;
use std::process::{Command, Output};

use crate::storage::{PackageConfig, PackageManagerType};

use super::dnf::DnfManger;
use super::zypper::ZypperManager;
use super::{ChangelogQuery, Error, PackageChangelogResult, PackageManager};
use super::error::Result;

pub fn get_package_manager<'a>(config: &'a PackageConfig) -> Result<Box<dyn PackageManager + 'a>> {
    match config.package_manager {
        Some(PackageManagerType::Zypper) => Ok(Box::new(ZypperManager { config })),
        Some(PackageManagerType::Dnf) => Ok(Box::new(DnfManger { config })),
        _ => Err(Error::UnsupportedPackageManager)
    }
}

pub fn run_shell_command<F>(command: &str, get_error: F) -> Result<()>
where F: Fn(String) -> Error {
    let output = Command::new("sh")
        .args(["-c", command])
        .output()?;

    process_cmd_output(output, get_error)?;

    Ok(())
}

pub fn process_cmd_output<F>(output: Output, get_error: F) -> Result<String>
where F: Fn(String) -> Error {
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(get_error(stderr))
    } else {
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    }
}

pub fn matches_query(name: &str, query: &str) -> bool {
    name.starts_with(query)
}

/** RPM functions **/

pub fn get_rpm_changelogs_result(query: &ChangelogQuery, path: &Path) -> Result<PackageChangelogResult> {
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

pub fn get_installed_pkg_timestamp(name: &str) -> Result<u64> {
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