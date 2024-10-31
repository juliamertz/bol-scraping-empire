use anyhow::Result;
use bytes::Bytes;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

/// this program is meant to be distributed to non-techical people
/// Automated updating makes the most sense for this usecase

#[derive(Debug, PartialEq, Eq)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl Version {
    fn parse(value: &str) -> Result<Self> {
        let split = value
            .strip_prefix("v")
            .unwrap_or(value)
            .split(".")
            .collect::<Vec<_>>();

        match split.as_slice() {
            [major, minor, patch] => Ok(Self {
                major: major.parse()?,
                minor: minor.parse()?,
                patch: patch.parse()?,
            }),
            _ => anyhow::bail!("invalid version format"),
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let major = self.major.cmp(&other.major);
        let minor = self.minor.cmp(&other.minor);
        let patch = self.patch.cmp(&other.patch);

        major.then(minor.then(patch))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

//TODO: get repo uri from cargo.toml
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
        // github requires a user-agent header, otherwise it will respond with status 400
        .header(USER_AGENT, "Rust-Bol-Empire")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    let text = &res.text().await?;
    let data: Vec<Release> = serde_json::from_str(text)?;

    Ok(data)
}

async fn latest_release() -> Result<Release> {
    let releases = query_releases().await?;
    let latest = releases.into_iter().next().expect("At least one release");

    Ok(latest)
}

async fn fetch_latest_bin<'a>() -> Result<Bytes> {
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

    let res = reqwest::get(url).await.unwrap();

    Ok(res.bytes().await.unwrap())
}

fn extract_tar(bytes: Bytes, dst: &Path) -> Result<()> {
    let filename = dst.join("archive.tar.gz");
    std::fs::write(&filename, bytes)?;

    let output = Command::new("tar")
        .args([
            "xf",
            filename.to_str().unwrap(),
            &format!("--directory={}", dst.to_str().unwrap()),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        anyhow::bail!("Unable to extract archive, error: {}", stderr)
    }

    std::fs::remove_file(filename)?;

    Ok(())
}

fn replace_current_executable(bin: PathBuf) -> Result<()> {
    let bin_arg = std::env::args().next().expect("binary path argument");
    let bin_path = PathBuf::from_str(&bin_arg).expect("valid path");

    if !bin.exists() {
        anyhow::bail!("unable to copy non-existing binary at path: {bin:?}")
    }

    std::fs::copy(bin, bin_path)?;
    Ok(())
}

pub async fn try_update() -> Result<bool> {
    let latest = &latest_release().await?;
    let version = Version::parse(&latest.tag_name).unwrap();
    let current = Version::parse(env!("CARGO_PKG_VERSION")).expect("valid runtime version");

    match version.cmp(&current) {
        Ordering::Equal => {
            println!("Je gebruikt de nieuwste versie {current}");
            Ok(false)
        }
        Ordering::Less => {
            println!("Lokale versie {current} is nieuwer dan laatste release {version}, update wordt overgeslagen");
            Ok(false)
        }
        Ordering::Greater => {
            println!(
                "Lokale versie {current} is ouder dan laatste release {version}, attempting update"
            );

            println!("Fetching binaries...");
            let latest = fetch_latest_bin().await?;

            println!("Unpacking binaries...");
            let temp_dir = PathBuf::from_str("temp_unpack")?;
            std::fs::create_dir(&temp_dir)?;
            extract_tar(latest, &temp_dir)?;

            println!("Programma probeert nu zichzelf te vervangen met de nieuwe versie, er wordt verwacht dat het programma crasht wanneer dit lukt.");
            replace_current_executable(temp_dir.join(env!("CARGO_PKG_NAME")))?;

            Ok(true)
        }
    }
}
