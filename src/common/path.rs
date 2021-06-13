use url::Url;

use crate::{Error, Result};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

pub enum UserPath {
    Url(Url),
    File(PathBuf),
}

impl FromStr for UserPath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = match url::Url::parse(&s) {
            Ok(url) if url.scheme() == "file" => {
                let path = url
                    .to_file_path()
                    .map_err(|_| Error::BadPath(url.path().to_owned()))?;

                Self::File(path)
            }
            Ok(url) => Self::Url(url),
            _ => Self::File(PathBuf::from(s)),
        };

        Ok(normalized)
    }
}

impl Display for UserPath {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::File(f) => fmt.write_str(&f.to_string_lossy().to_string()),
            Self::Url(u) => fmt.write_str(&u.to_string()),
        }
    }
}
