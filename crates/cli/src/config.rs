use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uploader::bol;

static FILENAME: &str = "secrets.toml";
static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub bol: bol::Credentials,
}

pub fn read() -> Result<&'static Config> {
    if let Some(value) = CONFIG.get() {
        return Ok(value);
    }

    let file_path = std::env::current_dir()
        .context("valid current working directory")?
        .join(FILENAME);

    let data = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => anyhow::bail!(
            "bestand '{FILENAME}' kan niet gevonden worden in huidige folder, error: {err:?}"
        ),
    };

    let config = toml::from_str::<Config>(&data).context("unable to parse secrets")?;
    CONFIG.set(config).expect("Config to be unlocked");
    CONFIG.get().context("Config to be locked")
}
