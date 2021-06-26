use crate::common::atomic_save::{
    AtomicFile, AtomicSaveError, Durability, OverwriteBehavior,
};
use crate::common::Handler;
use crate::{Error, Result};
use mime::Mime;
use once_cell::sync::Lazy;
use pest::Parser;
use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

pub static APPS: Lazy<MimeApps> = Lazy::new(|| MimeApps::read().unwrap());

#[derive(Debug, Default, Clone, pest_derive::Parser)]
#[grammar = "common/ini.pest"]
pub struct MimeApps {
    pub(super) added_associations: HashMap<Mime, VecDeque<Handler>>,
    pub(super) removed_associations: HashMap<Mime, VecDeque<Handler>>,
    pub(super) default_apps: HashMap<Mime, VecDeque<Handler>>,
}

impl MimeApps {
    pub fn add_handler(&mut self, mime: Mime, handler: Handler) {
        self.default_apps
            .entry(mime)
            .or_default()
            .push_back(handler);
    }

    pub fn set_handler(&mut self, mime: Mime, handler: Handler) {
        self.default_apps.insert(mime, vec![handler].into());
    }

    pub fn remove_handler(&mut self, mime: &Mime) -> Result<()> {
        if let Some(_removed) = self.default_apps.remove(mime) {
            self.save()?;
        }

        Ok(())
    }

    pub fn path() -> Result<PathBuf> {
        let mut config = xdg::BaseDirectories::new()?.get_config_home();
        config.push("mimeapps.list");
        Ok(config)
    }
    pub fn read() -> Result<Self> {
        let raw_conf = {
            let mut buf = String::new();
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .read(true)
                .open(Self::path()?)?
                .read_to_string(&mut buf)?;
            buf
        };
        let file = Self::parse(Rule::file, &raw_conf)?.next().unwrap();

        let mut current_section_name = "".to_string();
        let mut conf = Self {
            added_associations: HashMap::default(),
            removed_associations: HashMap::default(),
            default_apps: HashMap::default(),
        };

        file.into_inner().for_each(|line| {
            match line.as_rule() {
                Rule::section => {
                    current_section_name = line.into_inner().concat();
                }
                Rule::property => {
                    let mut inner_rules = line.into_inner(); // { name ~ "=" ~ value }

                    let name = inner_rules.next().unwrap().as_str();
                    let handlers = {
                        use itertools::Itertools;

                        inner_rules
                            .next()
                            .unwrap()
                            .as_str()
                            .split(";")
                            .filter(|s| !s.is_empty())
                            .unique()
                            .filter_map(|s| Handler::from_str(s).ok())
                            .collect::<VecDeque<_>>()
                    };

                    if !handlers.is_empty() {
                        match (
                            Mime::from_str(name),
                            current_section_name.as_str(),
                        ) {
                            (Ok(mime), "Added Associations") => {
                                conf.added_associations.insert(mime, handlers)
                            }

                            (Ok(mime), "Removed Associations") => {
                                conf.removed_associations.insert(mime, handlers)
                            }

                            (Ok(mime), "Default Applications") => {
                                conf.default_apps.insert(mime, handlers)
                            }
                            _ => None,
                        };
                    }
                }
                _ => {}
            }
        });

        Ok(conf)
    }
    pub fn save(&self) -> Result<()> {
        use itertools::Itertools;
        use std::io::prelude::*;
        use std::io::BufWriter;

        let af = AtomicFile::new(
            &Self::path()?,
            OverwriteBehavior::AllowOverwrite,
            Durability::DontSyncDir,
        );
        af.write(|f| -> Result<()> {
            let mut writer = BufWriter::new(f);

            #[rustfmt::skip]
            let mut write_section = |
                title,
                items: &HashMap<Mime, VecDeque<Handler>>,
            | -> Result<()> {
                writer.write_all(title)?;
                for (k, v) in items.iter().sorted() {
                    writer.write_all(k.essence_str().as_ref())?;
                    writer.write_all(b"=")?;
                    writer.write_all(v.iter().join(";").as_ref())?;
                    writer.write_all(b";\n")?;
                }
                Ok(())
            };

            write_section(b"[Added Associations]\n", &self.added_associations)?;

            if !self.removed_associations.is_empty() {
                write_section(
                    b"\n[Removed Associations]\n",
                    &self.removed_associations,
                )?;
            }

            write_section(b"\n[Default Applications]\n", &self.default_apps)?;

            writer.flush()?;
            Ok(())
        })
        .map_err(|e| match e {
            AtomicSaveError::Internal(e) => Error::Io(e),
            AtomicSaveError::User(e) => e,
        })?;
        Ok(())
    }
    pub fn print(&self, detailed: bool) -> Result<()> {
        use itertools::Itertools;

        let to_rows = |map: &HashMap<Mime, VecDeque<Handler>>| {
            map.iter()
                .sorted()
                .map(|(k, v)| vec![k.to_string(), v.iter().join(", ")])
                .collect::<Vec<_>>()
        };

        let table = ascii_table::AsciiTable::default();

        if detailed {
            println!("Default Apps");
            table.print(to_rows(&self.default_apps));
            if !self.added_associations.is_empty() {
                println!("Added Associations");
                table.print(to_rows(&self.added_associations));
            }
        } else {
            table.print(to_rows(&self.default_apps));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<()> {
        Ok(())
    }
}
