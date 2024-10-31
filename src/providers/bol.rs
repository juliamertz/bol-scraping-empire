use super::*;

const RESULTS_PER_PAGE: usize = 24;

pub async fn query_products(url: &str, pages: usize) -> Result<Products> {
    let mut handles = Vec::with_capacity(pages);

    for i in 0..pages {
        let url = url.to_owned();
        let handle = tokio::spawn(async move {
            println!("querying page {}", i + 1);
            let url = paginate_url(&url, i + 1);
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
        if let Err(_err) = parse_product(element, &mut buffer) {
            // eprintln!("no parsey: {err:#}")
        }
    }

    buffer
}

lazy_static! {
    static ref container_selector: Selector =
        Selector::parse(".list-view.product-list.js_multiple_basket_buttons_page").unwrap();
    static ref image_selector: Selector = Selector::parse("img").unwrap();
    static ref title_selector: Selector = Selector::parse(".product-title").unwrap();
    static ref price_selector: Selector = Selector::parse(r#"meta[itemprop="price"]"#).unwrap();
    static ref price_old_selector: Selector =
        Selector::parse(r#"del[data-test="from-price"]"#).unwrap();
}

fn parse_product(el: ElementRef<'_>, buffer: &mut Vec<Product>) -> Result<()> {
    // TODO SVG IMAGES
    let image = el.select(&image_selector).next().context("Image source")?;
    let image = image.attr("src").unwrap_or(
        image
            .attr("data-src")
            .context("either src or data-src attr for img")?,
    );

    let title = el
        .select(&title_selector)
        .next()
        .expect("a title")
        .inner_html();

    let url = el
        .select(&title_selector)
        .next()
        .expect("a title")
        .attr("href")
        .expect("product to have url");

    let price = match el.select(&price_old_selector).next() {
        Some(price) => price.inner_html(),
        None => el
            .select(&price_selector)
            .next()
            .expect("a price")
            .attr("content")
            .expect("price content")
            .parse()
            .expect("valid f64 price"),
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
