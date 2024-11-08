use anyhow::Result;
use reqwest::Method;
use serde::{Deserialize, Serialize};

pub struct Secrets {
    pub api_key: String,
    pub client_secret: String,
}

impl Secrets {
    pub fn new(api_key: &str, client_secret: &str) -> Self {
        Self {
            api_key: api_key.into(),
            client_secret: client_secret.into(),
        }
    }
}

pub struct Client {
    secrets: Secrets,
}

impl Client {
    pub fn new(secrets: Secrets) -> Self {
        Self { secrets }
    }

    async fn request(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<Vec<u8>>,
    ) -> Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let url = format!("https://api.bol.com/retailer{}", endpoint);

        let req = client
            .request(method, url)
            .header(reqwest::header::ACCEPT, "application/vnd.retailer.v10+json")
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/vnd.retailer.v10+json",
            );

        let res = match body {
            Some(data) => req.body(data),
            None => req,
        }
        .send()
        .await
        .expect("valid");

        Ok(res)
    }

    pub async fn create_offer(
        &self,
        // offer: &Offer
    ) -> Result<()> {
        // let data = Some(serde_json::to_vec(offer)?);
        let data = None;
        let response = self.request(Method::POST, "/offer", data).await?;
        dbg!(&response);

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Offer {
    ean: String,
    condition: Condition,
    reference: String,
    on_hold_by_retailer: bool,
    unknown_product_title: String,
    pricing: Pricing,
    stock: Stock,
    fulfilment: Fulfilment,
}

#[derive(Debug, Deserialize, Serialize)]
struct Condition {
    name: String,
    category: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Pricing {
    bundle_prices: Vec<BundlePrice>,
}
#[derive(Debug, Deserialize, Serialize)]
struct BundlePrice {
    quantity: u32,
    unit_price: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Stock {
    amount: u32,
    managed_by_retailer: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Fulfilment {
    method: String,
    delivery_code: String,
}
