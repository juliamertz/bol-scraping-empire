use anyhow::Result;
use bytes::Bytes;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// this program is meant to be distributed to non-techical people
/// Automated updating makes the most sense for this usecase

#[derive(Debug)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl Version {
    pub fn parse(value: &str) -> Result<Self> {
        let split = value.split(".").collect::<Vec<_>>();

        match split.as_slice() {
            [major, minor, patch] => Ok(Self {
                major: major.parse()?,
                minor: minor.parse()?,
                patch: patch.parse()?,
            }),
            _ => anyhow::bail!("invalid version format"),
        }
    }

    pub fn current() -> Self {
        Self::parse(env!("CARGO_PKG_VERSION")).expect("valid runtime version")
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

static REPO: &str = "juliamertz/bol-scraping-empire";

#[derive(Serialize, Deserialize, Debug)]
pub struct Release {
    pub tag_name: String,
    pub url: String,
}

async fn query_releases() -> Result<Vec<Release>> {
    let url = format!("https://api.github.com/repos/{}/releases", REPO);

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, "Rust-Bol-Empire")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    let text = &res.text().await?;
    let data: Vec<Release> = serde_json::from_str(text)?;

    Ok(data)
}

pub async fn latest_release() -> Result<Release> {
    let releases = query_releases().await?;
    let latest = releases.into_iter().next().expect("At least one release");

    // let target = if cfg!(target_os = "linux") {
    //     "x86_64-unknown-linux-gnu"
    // } else if cfg!(target_os = "macos") {
    //     "x86_64-unknown-linux-gnu.tar.gz"
    // } else {
    //     anyhow::bail!("unsupported OS")
    // };

    Ok(latest)
}

pub async fn latest_version() -> Result<Version> {
    let latest = latest_release().await?;
    Version::parse(
        latest
            .tag_name
            .strip_prefix("v")
            .unwrap_or(&latest.tag_name),
    )
}

pub async fn fetch_latest_bin<'a>() -> Result<Bytes> {
    let name = env!("CARGO_PKG_NAME");
    let arch = std::env::consts::ARCH;

    let (vendor, kernel) = if cfg!(target_os = "linux") {
        ("unknown", "linux-gnu")
    } else if cfg!(target_os = "macos") {
        ("apple", "darwin")
    } else {
        anyhow::bail!("Unsupported OS")
    };

    let asset_filename = format!("{name}-{arch}-{vendor}-{kernel}.tar.gz");
    let url = format!(
        "https://github.com/{}/releases/latest/download/{}",
        REPO, asset_filename
    );

    let res = reqwest::get(url).await?;

    Ok(res.bytes().await?)
}
