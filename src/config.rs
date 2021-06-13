use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Config {}
    }
}

impl Config {
    pub fn load() -> Self {
        confy::load("handlr").unwrap()
    }
}
