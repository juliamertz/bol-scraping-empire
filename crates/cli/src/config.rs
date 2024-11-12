use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uploader::api::bol;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub bol: bol::Credentials,
}

pub fn initialize() -> Result<&'static Config> {
    if CONFIG.get().is_some() {
        anyhow::bail!("Config already initialized")
    }

    let file_path = std::env::current_dir()
        .context("valid current working directory")?
        .join("secrets.toml");

    let data = std::fs::read_to_string(file_path).context("unable to read secrets.toml")?;
    let ser = toml::from_str::<Config>(&data).context("unable to parse secrets")?;

    CONFIG.set(ser).expect("Config to be unlocked");
    CONFIG.get().context("Config to be locked")
}

pub fn read() -> &'static Config {
    CONFIG.get().expect("Config to be initialized")
}
