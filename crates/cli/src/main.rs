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

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    ask_location: bool,

    #[arg(long)]
    location: Option<PathBuf>,
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
    let _conf = config::initialize()?;

    let state = Status::new(|status| {
        let mut stdout = io::stdout();
        stdout.queue(cursor::MoveTo(0, 0)).expect("cursor to move");
        stdout
            .execute(terminal::Clear(terminal::ClearType::FromCursorDown))
            .expect("to clear terminal");
        write!(stdout, "{status}").expect("to write status into stdout");
    });

    // let client = uploader::bol::Client::new_with_session(&conf.bol).await?;
    // let offer = uploader::types::Offer::new("Belenthi Digitale Kindercamera - Fototoestel voor kinderen - Speelgoedcamera - HD 1080p - Incl. accessoires - Vlog camera kinderen - Roze", "8720938943591", 34.99, 1, None);
    // dbg!(&offer);
    // client.create_offer(&offer).await?;
    // std::process::exit(0);

    #[cfg(feature = "updater")]
    if let Err(err) = versioning::try_update().await {
        eprintln!(
            "Er ging iets fout tijdens het automatish updaten, error: {:?}",
            err
        )
    }

    let url = read_line("Link naar zoekresultaten")?;
    let pages = read_line("Hoeveel paginas? (1)")?.parse().unwrap_or(1);

    let provider = Provider::from_url(&url)?;
    let products = provider.query_products(&url, pages, state).await?;

    let mut workbook = Workbook::new();
    workbook.push_worksheet(products.as_worksheet()?);

    println!("Output excel sheet gereed...");

    let mut outfile = PathBuf::from_str(OUTFILE).unwrap();
    if let Some(path) = cli.location {
        outfile = path;
    } else if cli.ask_location {
        outfile = rfd::FileDialog::new()
            .set_file_name(OUTFILE)
            .save_file()
            .unwrap_or(outfile)
    }

    workbook.save(outfile)?;
    println!("Done!");

    Ok(())
}
