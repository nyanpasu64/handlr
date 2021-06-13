use config::CONFIG;
use error::{Error, Result};
use once_cell::sync::Lazy;

mod apps;
mod cli;
mod common;
mod config;
mod error;
mod utils;

fn main() -> Result<()> {
    use clap::Clap;
    use cli::Cmd;

    // create config if it doesn't exist
    Lazy::force(&CONFIG);

    let mut apps = (*apps::CANONICAL).clone();

    let res = || -> Result<()> {
        match Cmd::parse() {
            Cmd::Set { mime, handler } => {
                apps.set_handler(mime.0, handler);
                apps.save()?;
            }
            Cmd::Add { mime, handler } => {
                apps.add_handler(mime.0, handler);
                apps.save()?;
            }
            Cmd::List { all } => {
                apps.print(all)?;
            }
            Cmd::Unset { mime } => {
                apps.remove_handler(&mime.0)?;
            }
        }
        Ok(())
    }();

    match (res, atty::is(atty::Stream::Stdout)) {
        (Err(e), true) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        (Err(e), false) => {
            utils::notify("handlr error", &e.to_string())?;
            std::process::exit(1);
        }
        _ => Ok(()),
    }
}
