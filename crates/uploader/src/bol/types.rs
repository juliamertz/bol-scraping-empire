use std::{default, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
    pub ean: String,
    pub condition: Condition,
    pub reference: String,
    pub on_hold_by_retailer: bool,
    pub unknown_product_title: String,
    pub pricing: Pricing,
    pub stock: Stock,
    pub fulfilment: Fulfilment,
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

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Pricing {
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
pub struct Stock {
    amount: i32,
    managed_by_retailer: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "UPPERCASE")]
enum FulfilmentMethod {
    /// Fulfillment by retailer
    #[default]
    Fbr,
    /// Fulfillment by bol
    Fbb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DeliveryCode {
    U24Uurs23,
    U24Uurs22,
    U24Uurs21,
    U24Uurs20,
    U24Uurs19,
    U24Uurs18,
    U24Uurs17,
    U24Uurs16,
    U24Uurs15,
    U24Uurs14,
    U24Uurs13,
    U24Uurs12,
    D1To2,
    D2To3,
    D3To5,
    D4To8,
    D1To8,
    MijnLeverbelofte,
    VVB,
}

impl FromStr for DeliveryCode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "24uurs-23" => Self::U24Uurs23,
            "24uurs-22" => Self::U24Uurs22,
            "24uurs-21" => Self::U24Uurs21,
            "24uurs-20" => Self::U24Uurs20,
            "24uurs-19" => Self::U24Uurs19,
            "24uurs-18" => Self::U24Uurs18,
            "24uurs-17" => Self::U24Uurs17,
            "24uurs-16" => Self::U24Uurs16,
            "24uurs-15" => Self::U24Uurs15,
            "24uurs-14" => Self::U24Uurs14,
            "24uurs-13" => Self::U24Uurs13,
            "24uurs-12" => Self::U24Uurs12,
            "1-2d" => Self::D1To2,
            "2-3d" => Self::D2To3,
            "3-5d" => Self::D3To5,
            "4-8d" => Self::D4To8,
            "1-8d" => Self::D1To8,
            "MijnLeverbelofte" => Self::MijnLeverbelofte,
            "VVB" => Self::VVB,
            _ => anyhow::bail!("Invalid delivery code: {s}"),
        })
    }
}

impl ToString for DeliveryCode {
    fn to_string(&self) -> String {
        match self {
            Self::U24Uurs23 => "24uurs-23",
            Self::U24Uurs22 => "24uurs-22",
            Self::U24Uurs21 => "24uurs-21",
            Self::U24Uurs20 => "24uurs-20",
            Self::U24Uurs19 => "24uurs-19",
            Self::U24Uurs18 => "24uurs-18",
            Self::U24Uurs17 => "24uurs-17",
            Self::U24Uurs16 => "24uurs-16",
            Self::U24Uurs15 => "24uurs-15",
            Self::U24Uurs14 => "24uurs-14",
            Self::U24Uurs13 => "24uurs-13",
            Self::U24Uurs12 => "24uurs-12",
            Self::D1To2 => "1-2d",
            Self::D2To3 => "2-3d",
            Self::D3To5 => "3-5d",
            Self::D4To8 => "4-8d",
            Self::D1To8 => "1-8d",
            Self::MijnLeverbelofte => "MijnLeverbelofte",
            Self::VVB => "VVB",
        }
        .to_string()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fulfilment {
    method: FulfilmentMethod,
    delivery_code: DeliveryCode,
    // string, Enum: "24uurs-23" "24uurs-22" "24uurs-21" "24uurs-20" "24uurs-19" "24uurs-18" "24uurs-17" "24uurs-16" "24uurs-15" "24uurs-14" "24uurs-13" "24uurs-12" "1-2d" "2-3d" "3-5d" "4-8d" "1-8d" "MijnLeverbelofte" "VVB"
    // delivery_code: String,
}

impl Offer {
    pub fn new(
        title: &str,
        ean: &str,
        price: f64,
        stock: i32,
        reference: Option<&str>,
        delivery_code: DeliveryCode,
    ) -> Self {
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
                method: FulfilmentMethod::default(),
                delivery_code,
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

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    language: Language,
    attributes: Vec<Attribute>,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    id: String,
    values: Vec<AttributeValue>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeValue {
    value: String,
    unit_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    url: String,
    labels: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Nl,
    // NlBe,
    // Fr,
    // FrBe,
}

pub mod responses {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateOffer {
        process_status_id: Option<String>,
        entity_id: Option<String>,
        event_type: String,
        description: String,
        error_message: Option<String>,
        create_timestamp: String,
        links: Vec<OfferLink>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct OfferLink {
        rel: String,
        href: String,
        hreflang: String,
        media: String,
        title: String,
        r#type: String,
        deprecation: String,
        profile: String,
        name: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum OfferCreationStatus {
        Pending,
        Success,
        Failure,
        Timeout,
    }
}
