mod providers;

use anyhow::Result;
use providers::{amazon, bol};
use rust_xlsxwriter::Workbook;
use std::io::{self, BufRead};

fn read_line(msg: &str) -> std::io::Result<String> {
    println!("{msg}:");
    let stdin = io::stdin().lock();
    stdin.lines().next().expect("input")
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args();
    let subcommand = args.next().expect("a subcommand");

    let url = read_line("Link naar amazon zoekresultaten")?;
    let pages = read_line("Hoeveel paginas")?.parse().expect("Valid usize");

    let products = match subcommand.to_lowercase().as_str() {
        "bol" => bol::query_products(&url, pages).await?,
        "amazon" => amazon::query_products(&url, pages).await?,

        _ => anyhow::bail!("choose from: [bol, amazon]"),
    };

    let mut workbook = Workbook::new();
    workbook.push_worksheet(products.as_worksheet()?);
    workbook.save("products.xlsx")?;
    println!("Done!");

    Ok(())
}
