pub mod amazon;
pub mod bol;

pub use anyhow::{Context, Result};
pub use bol_scraper_empire::fetch_dom;
pub use lazy_static::lazy_static;
pub use regex::Regex;
pub use scraper::{selectable::Selectable, ElementRef, Html, Selector};

use std::ops::{Deref, DerefMut};

#[derive(clap::Subcommand, Debug)]
pub enum Provider {
    Amazon,
    Bol,
}

impl Provider {
    pub async fn query_products(&self, url: &str, pages: usize) -> Result<Products> {
        match self {
            Self::Amazon => amazon::query_products(url, pages).await,
            Self::Bol => bol::query_products(url, pages).await,
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
    pub fn as_worksheet(&self) -> Result<rust_xlsxwriter::Worksheet> {
        let mut worksheet = rust_xlsxwriter::Worksheet::new();

        let column_names = ["title", "image", "url", "price"];
        for (col, name) in column_names.iter().enumerate() {
            worksheet.write(0, col as u16, *name)?;
        }

        for (i, product) in self.0.iter().enumerate() {
            let row = (i + 1) as u32;
            worksheet.write(row, 0, &product.title)?;
            worksheet.write(row, 1, &product.image)?;
            worksheet.write(row, 2, &product.url)?;
            worksheet.write(row, 3, product.price)?;
        }

        Ok(worksheet)
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
