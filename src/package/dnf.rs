use std::path::Path;
use std::process::Command;

use regex::Regex;

use crate::storage::PackageConfig;

use super::{utilities, ChangelogQuery, Error, PackageChangelogResult, PackageManager, PackageUpdateItem};
use super::error::Result;

pub struct DnfManger<'a> {
    pub config: &'a PackageConfig
}

impl<'a> PackageManager for DnfManger<'a> {
    fn get_config(&self) -> &PackageConfig {
        self.config
    }

    fn get_package_changelogs_result(&self, query: &ChangelogQuery, path: &Path) -> Result<PackageChangelogResult> {
        utilities::get_rpm_changelogs_result(query, path)
    }

    fn check_update(&self) -> Result<Vec<PackageUpdateItem>> {
        let output = Command::new("dnf")
            .arg("check-update")
            .output()?;
        let cmd_result = utilities::process_cmd_output(output, |err| Error::DnfError(err));

        match cmd_result {
            Ok(stdout) => {
                let regex = Regex::new(r"(?gm)^(\S+)\s+(\S+)\s+updates$")?;
        
                let items = regex.captures_iter(&stdout).map(|c| {
                    let (_, [name, version]) = c.extract();
                    PackageUpdateItem { name: name.to_owned(), new_version: Some(version.to_owned()), old_version: None}
                })
                .collect::<Vec<PackageUpdateItem>>();
        
                Ok(items)
            }
            _ => Ok(vec![])
        }
    }
}
