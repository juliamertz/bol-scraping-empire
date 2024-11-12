use std::fmt::{Display, Write};

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

impl Display for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = format!("{}:{}", self.client_id, self.client_secret);
        let encoded = base64::prelude::BASE64_STANDARD.encode(token);
        f.write_str(&encoded)
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

    pub async fn new_with_session(creds: &Credentials) -> Result<Self> {
        let mut client = Self::new();
        client.authenticate(creds).await?;
        Ok(client)
    }

    pub async fn authenticate(&mut self, creds: &Credentials) -> Result<()> {
        let res = reqwest::Client::new()
            .post("https://login.bol.com/token?grant_type=client_credentials")
            .header(header::CONTENT_LENGTH, 0)
            .header(header::AUTHORIZATION, format!("Basic {}", creds))
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

        dbg!(&req);

        let res = match body {
            Some(data) => req.body(data),
            None => req,
        }
        .send()
        .await
        .expect("valid");

        Ok(res)
    }

    pub async fn create_offer(&self, offer: &Offer) -> Result<()> {
        let data = Some(serde_json::to_vec(offer)?);
        let res = self.request(Method::POST, "/offer", data).await?;

        dbg!(&res);
        dbg!(&res.text().await?);
        // if res.status() != StatusCode::ACCEPTED {
        //     anyhow::bail!("Expected status 202 Accepted got {}", res.status())
        // }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
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
#[serde(rename_all = "UPPERCASE")]
pub enum ConditionName {
    New,
    AsNew,
    Good,
    Reasonable,
    Moderate,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConditionCategory {
    New,
    SecondHand,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    name: ConditionName,
    category: ConditionCategory,
    comment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Pricing {
    bundle_prices: Vec<BundlePrice>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BundlePrice {
    quantity: u32,
    unit_price: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Stock {
    amount: i32,
    managed_by_retailer: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum FulfilmentMethod {
    Fbr,
    Fbb,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Fulfilment {
    method: FulfilmentMethod,
    /// string, Enum: "24uurs-23" "24uurs-22" "24uurs-21" "24uurs-20" "24uurs-19" "24uurs-18" "24uurs-17" "24uurs-16" "24uurs-15" "24uurs-14" "24uurs-13" "24uurs-12" "1-2d" "2-3d" "3-5d" "4-8d" "1-8d" "MijnLeverbelofte" "VVB"
    delivery_code: String,
}

impl Offer {
    pub fn new(title: &str, ean: &str, price: f64, stock: i32, reference: Option<&str>) -> Self {
        Self {
            ean: ean.to_string(),
            pricing: Pricing::new(price),
            condition: Condition::default(),
            reference: reference.unwrap_or_default().to_string(),
            on_hold_by_retailer: false,
            stock: Stock {
                amount: stock,
                managed_by_retailer: false,
            },
            unknown_product_title: title.to_string(),
            fulfilment: Fulfilment {
                method: FulfilmentMethod::Fbr,
                // TODO:
                delivery_code: "24uurs-23".into(),
            },
        }
    }
}

impl Default for Condition {
    fn default() -> Self {
        Self {
            name: ConditionName::New,
            category: ConditionCategory::New,
            comment: None,
        }
    }
}

impl Pricing {
    fn new(unit_price: f64) -> Self {
        Self {
            bundle_prices: vec![BundlePrice {
                quantity: 1,
                unit_price,
            }],
        }
    }
}
