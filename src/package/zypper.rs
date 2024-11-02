use std::path::Path;
use std::process::Command;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::storage::PackageConfig;

use super::{utilities, ChangelogQuery, Error, PackageChangelogResult, PackageManager, PackageUpdateItem};
use super::error::Result;

pub struct ZypperManager<'a> {
    pub config: &'a PackageConfig
}

impl<'a> PackageManager for ZypperManager<'a> {

    fn get_config(&self) -> &PackageConfig {
        self.config
    }

    fn get_package_changelogs_result(&self, query: &ChangelogQuery, path: &Path) -> Result<PackageChangelogResult> {
        utilities::get_rpm_changelogs_result(query, path)
    }

    fn check_update(&self) -> Result<Vec<PackageUpdateItem>> {
        let output = Command::new("zypper")
            .args(["--xmlout", "lu"])
            .output()?;
        let stdout = utilities::process_cmd_output(output, |err| Error::ZypperError(err))?;
        let mut reader = Reader::from_str(stdout.as_str());
        let mut items = Vec::new();

        loop {
            match reader.read_event()? {
                Event::Start(e) if e.name().as_ref() == b"update" =>{
                    let mut name: String = String::new();
                    let mut edition: Option<String> = None;
                    let mut edition_old: Option<String> = None;
                    let attributes = e.attributes();

                    for attr_result in attributes {
                        let attr = attr_result?;

                        match attr.key.as_ref() {
                            b"name" => name = attr_to_string(attr),
                            b"edition" => edition = Some(attr_to_string(attr)),
                            b"edition-old" => edition_old = Some(attr_to_string(attr)),
                            _ => ()
                        }
                    }

                    if !name.is_empty() {
                        items.push(PackageUpdateItem { name, new_version: edition, old_version: edition_old });
                    }
                },
                Event::Eof => break,
                _ => ()
            }
        }

        Ok(items)
    }
}

fn attr_to_string(attr: Attribute) -> String {
    String::from_utf8_lossy(attr.value.as_ref()).to_string()
}