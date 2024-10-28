use anyhow::{Context, Result};
use bol_scraper_empire::fetch_dom;
use get_fields::GetFields;
use lazy_static::lazy_static;
use regex::Regex;
use rust_xlsxwriter::Worksheet;
use scraper::{selectable::Selectable, ElementRef, Html, Selector};

#[derive(Debug, GetFields)]
pub struct Product {
    pub title: String,
    pub image: String,
    pub url: String,
    pub price: f64,
}

#[derive(Debug)]
pub struct Products(Vec<Product>);

const RESULTS_PER_PAGE: usize = 56;

pub async fn query_products(url: &str, pages: usize) -> Result<Products> {
    let mut handles = Vec::with_capacity(pages);

    for i in 0..pages {
        let url = url.to_owned();
        let handle = tokio::spawn(async move {
            println!("querying page {}", i + 1);
            let url = paginate_url(&url, i);
            let doc = fetch_dom(&url).await.expect("valid dom");
            parse_products(doc)
        });

        handles.push(handle);
    }

    let results = futures::future::join_all(handles)
        .await
        .into_iter()
        .flat_map(|res| res.unwrap())
        .collect::<Vec<_>>();

    Ok(Products(results))
}

fn parse_products(doc: Html) -> Vec<Product> {
    let container = doc.select(&container_selector).next().unwrap();

    let mut buffer = Vec::with_capacity(RESULTS_PER_PAGE);
    for element in container.child_elements() {
        match element.attr("data-component-type") {
            Some("s-search-result") => {
                if parse_product(element, &mut buffer).is_err() {
                    continue;
                }
            }
            _ => continue,
        }
    }

    buffer
}

lazy_static! {
    static ref container_selector: Selector =
        Selector::parse(".s-main-slot.s-result-list.s-search-results").unwrap();
    static ref image_selector: Selector = Selector::parse(".s-image").unwrap();
    static ref title_wrapper_selector: Selector =
        Selector::parse(".s-title-instructions-style a").unwrap();
    static ref title_selector: Selector = Selector::parse("span").unwrap();
    static ref price_whole_selector: Selector = Selector::parse(".a-price-whole").unwrap();
    static ref price_fraction_selector: Selector = Selector::parse(".a-price-fraction").unwrap();
    static ref price_old_selector: Selector = Selector::parse(".a-price.a-text-price").unwrap();
}

fn parse_product(el: ElementRef<'_>, buffer: &mut Vec<Product>) -> Result<()> {
    let image = el
        .select(&image_selector)
        .next()
        .expect("product image")
        .attr("src")
        .expect("product image source");
    let title_wrapper = el
        .select(&title_wrapper_selector)
        .next()
        .expect("a title wrapper");
    let title = title_wrapper
        .select(&title_selector)
        .next()
        .expect("a title")
        .inner_html();

    if sponsored_regex.is_match(&title) {
        anyhow::bail!("Sponsored product");
    }

    let url = title_wrapper.attr("href").expect("product to have url");

    let price = match el.select(&price_old_selector).next() {
        Some(price) => price
            .child_elements()
            .nth(1)
            .unwrap()
            .inner_html()
            .strip_prefix("€")
            .expect("price to have euro symbol prefix")
            .to_string(),
        None => {
            let price_whole = el
                .select(&price_whole_selector)
                .next()
                .context("Expected a whole price")?
                .text()
                .next()
                .context("Expected a whole price")?;
            let price_fraction = el
                .select(&price_fraction_selector)
                .next()
                .context("Expected a price fraction")?
                .inner_html();

            format!("{},{}", price_whole, price_fraction)
        }
    };
    let price: f64 = price
        .replace(",", ".")
        .parse()
        .expect("Expected valid parsable floating point price");

    let product = Product {
        title,
        price,
        image: image.to_string(),
        url: url.to_string(),
    };

    buffer.push(product);
    Ok(())
}

lazy_static! {
    static ref sponsored_regex: Regex = Regex::new(r"Gesponsord").unwrap();
    static ref page_param_regex: Regex = Regex::new(r"page=\d*").unwrap();
}

fn paginate_url(url: &str, page: usize) -> String {
    if !page_param_regex.is_match(url) {
        format!("{}&page={}", url, page)
    } else {
        page_param_regex
            .replace(url, format!("page={}", page).as_str())
            .to_string()
    }
}

impl Products {
    pub fn as_worksheet(&self) -> Result<Worksheet> {
        let mut worksheet = Worksheet::new();

        for (col, name) in Product::get_fields.iter().enumerate() {
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
