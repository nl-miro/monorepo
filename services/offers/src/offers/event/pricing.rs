use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;

use crate::offers::{retailer_offer_id::RetailerOfferId, state::Price};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferPriceChangedEvent {
    pub id: RetailerOfferId,
    pub previous_price: Price,
    pub price: Price,
    pub updated_at: DateTime<Utc>,
}

impl OfferPriceChangedEvent {
    pub fn new(
        id: RetailerOfferId,
        previous_price: Price,
        price: Price,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            previous_price,
            price,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferPriceChangedEvent {
    fn event_type(&self) -> &'static str {
        "OfferPriceChangedNextGeneration"
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferPriceConfirmedEvent {
    pub id: RetailerOfferId,
    pub price: Price,
    pub updated_at: DateTime<Utc>,
}

impl OfferPriceConfirmedEvent {
    pub fn new(id: RetailerOfferId, price: Price, updated_at: DateTime<Utc>) -> Self {
        Self {
            id,
            price,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferPriceConfirmedEvent {
    fn event_type(&self) -> &'static str {
        "OfferPriceConfirmed"
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferHistoricPriceAddedEvent {
    pub id: RetailerOfferId,
    pub historic_price: Price,
    pub updated_at: DateTime<Utc>,
}

impl OfferHistoricPriceAddedEvent {
    pub fn new(id: RetailerOfferId, historic_price: Price, updated_at: DateTime<Utc>) -> Self {
        Self {
            id,
            historic_price,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferHistoricPriceAddedEvent {
    fn event_type(&self) -> &'static str {
        "OfferHistoricPriceAdded"
    }
}
