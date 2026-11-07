use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;
use retailer_sauce::{availability::Availability, item_condition::ItemCondition};

use crate::offers::{
    retailer_offer_id::RetailerOfferId,
    state::{Content, ExternalOfferIdentifiers, Price},
    target_market::TargetMarket,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferCreatedEvent {
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub url: String,
    pub identifiers: ExternalOfferIdentifiers,
    pub target_market: TargetMarket,
    pub language: String,
    pub content: Content,
    pub price: Price,
    pub availability: Option<Availability>,
    pub item_condition: Option<ItemCondition>,
    pub created_at: DateTime<Utc>,
}

impl ApplicationEvent for OfferCreatedEvent {
    fn event_type(&self) -> &'static str {
        "OfferCreatedNextGeneration"
    }
}

impl OfferCreatedEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RetailerOfferId,
        retailer_code: RetailerCode,
        url: String,
        identifiers: ExternalOfferIdentifiers,
        target_market: TargetMarket,
        language: String,
        content: Content,
        price: Price,
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
        }
    }
}
