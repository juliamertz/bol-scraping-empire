mod providers;

use anyhow::Result;
use providers::{amazon, bol};
use rust_xlsxwriter::Workbook;
use std::io::{self, BufRead};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    ask_location: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Amazon,
    Bol,
}

fn read_line(msg: &str) -> std::io::Result<String> {
    println!("{msg}:");
    let stdin = io::stdin().lock();
    stdin.lines().next().expect("input")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // let url = read_line("Link naar amazon zoekresultaten")?;
    // let pages = read_line("Hoeveel paginas")?.parse().expect("Valid usize");

    // let products = match subcommand.to_lowercase().as_str() {
    //     "bol" => bol::query_products(&url, pages).await?,
    //     "amazon" => amazon::query_products(&url, pages).await?,
    //
    //     _ => anyhow::bail!("choose from: [bol, amazon]"),
    // };
    //
    // let mut workbook = Workbook::new();
    // workbook.push_worksheet(products.as_worksheet()?);
    // workbook.save("products.xlsx")?;
    // println!("Done!");

    Ok(())
}
