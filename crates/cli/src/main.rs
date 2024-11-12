mod config;
use scraping::{self, providers::Provider, status::Status};
#[cfg(feature = "updater")]
mod versioning;

use anyhow::Result;
use rust_xlsxwriter::Workbook;
use std::{
    io::{self, BufRead},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};
use uploader::api::bol::Offer;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, default_value = "true")]
    ask_location: bool,
}

fn read_line(msg: &str) -> std::io::Result<String> {
    println!("{msg}:");
    let stdin = io::stdin().lock();
    stdin.lines().next().expect("input")
}

static OUTFILE: &str = "products.xlsx";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let state = Arc::new(Status::new());

    let conf = config::initialize()?;
    // let mut client = api::bol::Client::new();
    // client.authenticate(&conf.bol).await?;

    let offer = Offer::new("30inch dildo", "231231", 100.00, 1, None);
    dbg!(offer);
    // client.create_offer().await?;

    std::process::exit(0);

    #[cfg(feature = "updater")]
    if let Err(err) = versioning::try_update().await {
        eprintln!(
            "Er ging iets fout tijdens het automatish updaten, error: {:?}",
            err
        )
    }

    let url = read_line("Link naar zoekresultaten")?;
    let pages = read_line("Hoeveel paginas")?.parse().unwrap_or(1);

    let provider = Provider::from_url(&url)?;
    let products = provider.query_products(&url, pages, state).await?;

    let mut workbook = Workbook::new();
    workbook.push_worksheet(products.as_worksheet()?);

    println!("Output excel sheet gereed...");

    let mut outfile = PathBuf::from_str(OUTFILE).unwrap();
    if cli.ask_location {
        outfile = rfd::FileDialog::new()
            .set_file_name(OUTFILE)
            .save_file()
            .unwrap_or(outfile)
    }

    workbook.save(outfile)?;
    println!("Done!");

    Ok(())
}
