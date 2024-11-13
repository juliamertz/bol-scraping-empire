use serde::{Deserialize, Serialize};

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
