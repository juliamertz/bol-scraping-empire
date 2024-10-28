use anyhow::Result;
use reqwest::StatusCode;
use scraper::Html;

pub async fn fetch_dom(url: &str) -> Result<Html> {
    let res = reqwest::get(url).await?;
    if res.status() != StatusCode::OK {
        anyhow::bail!("Error while fetching DOM, got status {:?}", res.status())
    }

    let body = res.text().await?;

    Ok(Html::parse_document(&body))
}
