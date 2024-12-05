mod config;
use scraping::{self, providers::Provider, status::Status};
#[cfg(feature = "updater")]
mod versioning;

use anyhow::Result;
use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};
use rust_xlsxwriter::Workbook;
use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
    str::FromStr,
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scrape {
        #[arg(long)]
        ask_location: bool,

        #[arg(long)]
        url: Option<String>,
    },
    Upload,
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

    #[cfg(feature = "updater")]
    if let Err(err) = versioning::try_update().await {
        eprintln!(
            "Er ging iets fout tijdens het automatish updaten, error: {:?}",
            err
        )
    }

    match cli.command {
        Commands::Scrape { url, ask_location } => handle_scrape_command(url, ask_location).await?,
        Commands::Upload => handle_upload_command().await?,
    };

    Ok(())
}

async fn handle_upload_command() -> Result<()> {
    use uploader::bol::{types::Offer, Client};

    let conf = config::read()?;
    let client = Client::new_with_session(&conf.bol).await?;

    let offer = Offer::new(
        "Comfort",
        "9789059564169",
        999.99,
        10,
        Some("REF12345"),
    );

    // let ser = serde_json::to_string_pretty(&offer)?;
    // std::fs::write("./offer.json", ser)?;
    client.create_offer(&offer).await?;

    // client.get_orders().await?;

    Ok(())
}

async fn handle_scrape_command(url: Option<String>, ask_location: bool) -> Result<()> {
    dbg!(&url);
    let state = Status::new(|status| {
        let mut stdout = io::stdout();
        stdout.queue(cursor::MoveTo(0, 0)).expect("cursor to move");
        stdout
            .execute(terminal::Clear(terminal::ClearType::FromCursorDown))
            .expect("to clear terminal");
        write!(stdout, "{status}").expect("to write status into stdout");
    });

    let url = match url {
        Some(value) => value,
        None => read_line("Link naar zoekresultaten")?,
    };
    let pages = read_line("Hoeveel paginas? (1)")?.parse().unwrap_or(1);

    let provider = Provider::from_url(&url)?;
    let products = provider.query_products(&url, pages, state).await?;

    let mut workbook = Workbook::new();
    workbook.push_worksheet(products.as_worksheet()?);

    println!("Output excel sheet gereed...");

    let mut outfile = PathBuf::from_str(OUTFILE).unwrap();
    if ask_location {
        outfile = rfd::FileDialog::new()
            .set_file_name(OUTFILE)
            .save_file()
            .unwrap_or(outfile)
    }

    workbook.save(outfile)?;
    println!("Done!");

    Ok(())
}

