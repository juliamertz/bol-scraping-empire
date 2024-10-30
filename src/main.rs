mod providers;

use anyhow::Result;
use providers::{amazon, bol};
use rust_xlsxwriter::Workbook;
use std::{
    io::{self, BufRead},
    path::PathBuf,
    str::FromStr,
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    ask_location: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Amazon,
    Bol,
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

    let url = read_line(&format!("Link naar {:?} zoekresultaten", cli.command))?;
    let pages = read_line("Hoeveel paginas")?.parse().unwrap_or(1);

    let products = match cli.command {
        Commands::Bol => bol::query_products(&url, pages).await?,
        Commands::Amazon => amazon::query_products(&url, pages).await?,
    };

    let mut workbook = Workbook::new();
    workbook.push_worksheet(products.as_worksheet()?);

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
