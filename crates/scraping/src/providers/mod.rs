pub mod amazon;
pub mod bol;

pub use anyhow::{Context, Result};
pub use lazy_static::lazy_static;
pub use regex::Regex;
pub use scraper::{selectable::Selectable, ElementRef, Html, Selector};
use umya_spreadsheet::{Spreadsheet, Worksheet};

use crate::status;
use reqwest::StatusCode;
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

pub async fn fetch_dom(url: &str) -> Result<Html> {
    let res = reqwest::get(url).await?;
    if res.status() != StatusCode::OK {
        anyhow::bail!("Error while fetching DOM, got status {:?}", res.status())
    }

    let body = res.text().await?;

    Ok(Html::parse_document(&body))
}

#[derive(clap::Subcommand, Debug)]
pub enum Provider {
    Amazon,
    Bol,
}

impl Provider {
    pub async fn query_products(
        &self,
        url: &str,
        pages: usize,
        state: status::State,
    ) -> Result<Products> {
        match self {
            Self::Amazon => amazon::query_products(url, pages, state).await,
            Self::Bol => bol::query_products(url, pages, state).await,
        }
    }

    pub fn from_url(url: &str) -> Result<Self> {
        let domain = match url.split("/").nth(2) {
            Some(domain) => domain,
            None => anyhow::bail!("Invalid url, unable to parse domain"),
        };

        let parts = domain
            .split(".")
            .skip_while(|x| x == &"www")
            .collect::<Vec<_>>();

        let (provider, tld) = match parts.as_slice() {
            ["amazon", tld] => (Self::Amazon, tld),
            ["bol", tld] => (Self::Bol, tld),
            _ => anyhow::bail!("Unsupported url: {url}"),
        };

        if *tld != "com" && *tld != "nl" {
            anyhow::bail!("unsupported top level domain {tld}")
        }

        Ok(provider)
    }
}

#[derive(Debug)]
pub struct Product {
    pub title: String,
    pub image: String,
    pub url: String,
    pub price: f64,
    pub ean: Option<u64>,
}

#[derive(Debug)]
pub struct Products(Vec<Product>);

lazy_static! {
    static ref page_param_regex: Regex = Regex::new(r"page=\d*").unwrap();
}

pub fn paginate_url(url: &str, page: usize) -> String {
    if !url.contains("?") {
        return format!("{}?page={}", url, page);
    }

    if !page_param_regex.is_match(url) {
        format!("{}&page={}", url, page)
    } else {
        page_param_regex
            .replace(url, format!("page={}", page).as_str())
            .to_string()
    }
}

impl Products {
    pub fn from_spreadsheet(path: impl AsRef<Path>) -> Result<Self> {
        // let mut sheet = umya_spreadsheet::new_file();
        //
        // rust_xlsxwriter::Workbook::rea
        todo!()
    }

    pub fn as_spreadsheet(&self) -> Result<Spreadsheet> {
        let mut book = umya_spreadsheet::new_file();
        let sheet = book
            .get_sheet_mut(&0)
            .expect("workbook to have atleast one sheet");

        let column_names = ["title", "image", "url", "price", "ean"];
        for (col, name) in column_names.iter().enumerate() {
            sheet.get_cell_mut((0, col as u32)).set_value(*name);
        }

        for (i, product) in self.0.iter().enumerate() {
            let row = (i + 1) as u32;

            sheet.get_cell_mut((row, 0)).set_value(&product.title);
            sheet.get_cell_mut((row, 1)).set_value(&product.image);
            sheet.get_cell_mut((row, 2)).set_value(&product.url);
            sheet.get_cell_mut((row, 3)).set_value_number(product.price);
            if let Some(ean) = product.ean {
                sheet.get_cell_mut((row, 4)).set_value_number(ean as f64);
            }
        }

        Ok(book)
    }
}

impl From<Vec<Product>> for Products {
    fn from(value: Vec<Product>) -> Self {
        Self(value)
    }
}

impl Deref for Products {
    type Target = Vec<Product>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Products {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
