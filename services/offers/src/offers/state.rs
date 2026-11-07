use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use money::Money;
use retailer_sauce::availability::Availability;
use retailer_sauce::item_condition::ItemCondition;

use super::retailer_offer_id::RetailerOfferId;
use super::target_market::TargetMarket;

#[derive(Debug, PartialEq)]
pub struct RetailerOfferState {
    pub id: RetailerOfferId,
    retailer_code: RetailerCode,
    url: String,
    pub identifiers: ExternalOfferIdentifiers,
    pub target_market: TargetMarket,
    pub language: String,
    pub content: Content,
    pub price: Pricing,
    pub availability: Option<Availability>,
    pub item_condition: Option<ItemCondition>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RetailerOfferState {
    pub fn retailer_code(&self) -> &RetailerCode {
        &self.retailer_code
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

impl RetailerOfferState {
    // New states start with matching creation and update timestamps.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RetailerOfferId,
        retailer_code: RetailerCode,
        url: String,
        identifiers: ExternalOfferIdentifiers,
        target_market: TargetMarket,
        language: String,
        content: Content,
        price: Pricing,
        availability: Option<Availability>,
        item_condition: Option<ItemCondition>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            retailer_code,
            url,
            identifiers,
            target_market,
            language,
            content,
            price,
            availability,
            item_condition,
            created_at,
            updated_at: created_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExternalOfferIdentifiers {
    pub gtin: Option<String>,
    pub sku: Option<String>,
    pub shopify_product_gid: Option<String>,
    pub shopify_product_variant_gid: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Pricing {
    pub last: Price,
    pub history: Vec<Price>,
}

impl Pricing {
    pub fn new(last: Price) -> Self {
        Self {
            last,
            history: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Price {
    pub current: Money,
    pub previous: Option<Money>,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Content {
    pub title: String,
    pub description: String,
    pub images: Vec<String>,
}
