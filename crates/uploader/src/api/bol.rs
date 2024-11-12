use anyhow::Result;
use base64::Engine;
use reqwest::{header, Method, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
}

impl Credentials {
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AuthResponse {
    access_token: String,
}

#[derive(Default, Debug)]
pub struct Client {
    /// JWT token containing expiration etc...
    pub access_token: Option<String>,
}

static CONTENT_TYPE: &str = "application/vnd.retailer.v10+json";

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn authenticate(&mut self, creds: &Credentials) -> Result<()> {
        use base64::prelude::BASE64_STANDARD as base64;

        let client = reqwest::Client::new();
        let token = base64.encode(format!("{}:{}", creds.client_id, creds.client_secret));
        let res = client
            .post("https://login.bol.com/token?grant_type=client_credentials")
            .header(header::CONTENT_LENGTH, 0)
            .header(header::AUTHORIZATION, format!("Basic {token}"))
            .send()
            .await?;

        if res.status() != StatusCode::OK {
            anyhow::bail!(
                "non Ok status code when authenticating with bol: {}",
                res.status()
            )
        }

        let content = &res.text().await?;
        let data = serde_json::from_str::<AuthResponse>(content)?;
        self.access_token = Some(data.access_token);

        Ok(())
    }

    async fn request(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<Vec<u8>>,
    ) -> Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let url = format!("https://api.bol.com/retailer{}", endpoint);
        let access_token = match self.access_token {
            Some(ref token) => token,
            None => anyhow::bail!("Client is not authenticated."),
        };

        let req = client
            .request(method, url)
            .header(header::ACCEPT, CONTENT_TYPE)
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .header(header::CONTENT_TYPE, CONTENT_TYPE);

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
