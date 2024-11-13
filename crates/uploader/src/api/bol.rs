use super::types::*;

use anyhow::Result;
use base64::Engine;
use reqwest::{header, Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
}

impl Display for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = format!("{}:{}", self.client_id, self.client_secret);
        let encoded = base64::prelude::BASE64_STANDARD.encode(token);
        f.write_str(&encoded)
    }
}

#[derive(Serialize, Deserialize,Debug)]
struct AccessToken {
    access_token: String,
    token_type: String,
    expires_in: u32,
    scope: String,
}

#[derive(Default, Debug)]
pub struct Client {
    pub session: Option<AccessToken>,
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
        let data = serde_json::from_str::<AccessToken>(content)?;
        self.session = Some(data);

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
        let access_token = match self.session {
            Some(ref session) => &session.access_token,
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

    pub async fn create_product_content(&self, content: &Content) -> Result<()> {
        let data = Some(serde_json::to_vec(content)?);
        let res = self.request(Method::POST, "/content/products", data).await?;

        dbg!(&res);
        dbg!(&res.text().await?);
        // if res.status() != StatusCode::ACCEPTED {
        //     anyhow::bail!("Expected status 202 Accepted got {}", res.status())
        // }

        Ok(())
    }
}
