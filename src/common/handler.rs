use crate::{Error, Result};
use std::convert::TryFrom;
use std::ffi::OsString;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handler(OsString);

impl Display for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string_lossy())
    }
}

impl FromStr for Handler {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::resolve(s.into())
    }
}

impl Handler {
    pub fn assume_valid(name: OsString) -> Self {
        Self(name)
    }
    pub fn get_path(name: &std::ffi::OsStr) -> Option<PathBuf> {
        let mut path = PathBuf::from("applications");
        path.push(name);
        xdg::BaseDirectories::new().ok()?.find_data_file(path)
    }
    pub fn resolve(name: OsString) -> Result<Self> {
        let _path = Self::get_path(&name)
            .ok_or(Error::NotFound(name.to_string_lossy().into()))?;
        Ok(Self(name))
    }
}
