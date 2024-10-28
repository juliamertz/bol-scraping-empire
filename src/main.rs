mod amazon;

use anyhow::Result;
use rust_xlsxwriter::Workbook;
use std::io::{self, BufRead};

fn read_line(msg: &str) -> std::io::Result<String> {
    println!("{msg}:");
    let stdin = io::stdin().lock();
    stdin.lines().next().expect("input")
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = read_line("Link naar amazon zoekresultaten")?;
    let pages = read_line("Hoeveel paginas")?.parse().expect("Valid usize");

    let products = amazon::query_products(&url, pages).await?;

    let mut workbook = Workbook::new();
    workbook.push_worksheet(products.as_worksheet()?);
    workbook.save("products.xlsx")?;
    dbg!(products);

    Ok(())
}
