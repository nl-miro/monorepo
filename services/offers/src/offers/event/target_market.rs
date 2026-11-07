use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;

use crate::offers::{retailer_offer_id::RetailerOfferId, target_market::TargetMarket};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferTargetMarketChangedEvent {
    pub id: RetailerOfferId,
    pub previous_target_market: TargetMarket,
    pub target_market: TargetMarket,
    pub updated_at: DateTime<Utc>,
}

impl OfferTargetMarketChangedEvent {
    pub fn new(
        id: RetailerOfferId,
        previous_target_market: TargetMarket,
        target_market: TargetMarket,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            previous_target_market,
            target_market,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferTargetMarketChangedEvent {
    fn event_type(&self) -> &'static str {
        "OfferTargetMarketChanged"
    }
}
