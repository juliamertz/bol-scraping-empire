use super::*;

const RESULTS_PER_PAGE: usize = 24;

async fn query_specifications(product: Product, state: &status::Status) -> Product {
    state.add_pending();
    match query_product_page(&product.url).await {
        Ok(specifications) => {
            state.pending_success();
            Product {
                title: product.title,
                image: product.image,
                price: product.price,
                url: product.url,
                ean: Some(specifications.ean),
            }
        }
        Err(err) => {
            eprintln!("Error while trying to query product page: {err:?}");
            state.pending_errored();
            product
        }
    }
}

pub async fn query_products(url: &str, pages: usize, state: status::GlobalStatus) -> Result<Products> {
    let mut handles = Vec::with_capacity(pages);

    for i in 0..pages {
        let url = url.to_owned();
        let state = state.clone();
        let handle = tokio::spawn(async move {
            println!("querying page {}", i + 1);
            state.add_pending();
            let url = paginate_url(&url, i + 1);
            let doc = fetch_dom(&url).await.expect("valid dom");

            let products = parse_products(doc);
            state.pending_success();

            let with_specifications = products
                .into_iter()
                .map(|p| query_specifications(p, &state))
                .collect::<Vec<_>>();

            futures::future::join_all(with_specifications).await
        });

        handles.push(handle);
    }

    let results = futures::future::join_all(handles)
        .await
        .into_iter()
        .flat_map(|res| res.unwrap())
        .collect::<Vec<_>>();

    Ok(results.into())
}

async fn query_product_page(url: &str) -> Result<Specifications> {
    let doc = fetch_dom(url).await?;
    parse_product_page(doc)
}

fn parse_products(doc: Html) -> Vec<Product> {
    let container = doc.select(&container_selector).next().expect("Pagina komt niet overeen met de verwachte structuur. Deze is nog niet toegevoegd, of bol.com heeft hun pagina aangepast");

    let mut buffer = Vec::with_capacity(RESULTS_PER_PAGE);
    for element in container.child_elements() {
        if let Err(err) = parse_product_item(element, &mut buffer) {
            eprintln!("failed to parse product listing: {err:#}")
        }
    }

    buffer
}
lazy_static! {
    static ref specs_container_selector: Selector = Selector::parse("section[data-group-name='ProductSpecification'] .js_show-more-specifications .js_show-more-content").unwrap();
    static ref specs_section_selector: Selector = Selector::parse(".specs__list").unwrap();
    static ref specs_title_selector: Selector = Selector::parse(".specs__title").unwrap();
    static ref specs_value_selector: Selector = Selector::parse(".specs__value").unwrap();
}

#[derive(Debug)]
pub struct Specifications {
    pub ean: u64,
}

fn parse_product_page(doc: Html) -> Result<Specifications> {
    let specs = match doc.select(&specs_container_selector).next() {
        Some(specs) => specs,
        None => anyhow::bail!("No specs list"),
    };

    let mut ean_code: Option<u64> = None;

    'outer: for section in specs.child_elements() {
        if section.attr("class") != Some("specs") {
            continue;
        }
        let section = section
            .select(&specs_section_selector)
            .next()
            .context("Specifications wrapper in section")?;

        for item in section.child_elements() {
            let spec_title = item
                .select(&specs_title_selector)
                .next()
                .context("A specification title")?
                .inner_html();

            let spec_value = item
                .select(&specs_value_selector)
                .next()
                .context("A specification value")?
                .inner_html();

            if spec_title.trim() == "EAN" {
                ean_code = Some(spec_value.trim().parse()?);
                break 'outer;
            }
        }
    }

    let ean = match ean_code {
        Some(ean) => ean,
        None => anyhow::bail!("No EAN code found for product"),
    };

    Ok(Specifications { ean })
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

fn parse_product_item(el: ElementRef<'_>, buffer: &mut Vec<Product>) -> Result<()> {
    // TODO: SVG IMAGES
    let image = el.select(&image_selector).next().context("Image source")?;
    let image = image.attr("src").unwrap_or(
        image
            .attr("data-src")
            .context("either src or data-src attr for img")?,
    );

    let title = el
        .select(&title_selector)
        .next()
        .context("a title")?
        .inner_html();

    let url = el
        .select(&title_selector)
        .next()
        .context("a title")?
        .attr("href")
        .context("product to have url")?;

    for item in buffer.iter() {
        if item.url == url {
            anyhow::bail!("Product with same url alredy parsed")
        }
    }

    let price = match el.select(&price_old_selector).next() {
        Some(price) => price.inner_html(),
        None => el
            .select(&price_selector)
            .next()
            .context("Expected product to have a price")?
            .attr("content")
            .context("price content")?
            .parse()
            .context("valid f64 price")?,
    };
    let price: f64 = price
        .replace(",", ".")
        .parse()
        .context("Expected valid parsable floating point price")?;

    let product = Product {
        title,
        price,
        image: image.to_string(),
        url: format!("https://bol.com{}", url),
        ean: None,
    };

    buffer.push(product);
    Ok(())
}
