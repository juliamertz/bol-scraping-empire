use super::*;

const RESULTS_PER_PAGE: usize = 56;

pub async fn query_products(url: &str, pages: usize) -> QueryHandles<Product> {
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

    handles

    // let results = futures::future::join_all(handles)
    //     .await
    //     .into_iter()
    //     .flat_map(|res| res.unwrap())
    //     .collect::<Vec<_>>();
    //
    // Ok(Products(results))
}

pub fn parse_products(doc: Html) -> Vec<Product> {
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
}
