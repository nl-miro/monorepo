use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;
use retailer_sauce::availability::Availability;

use crate::offers::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferAvailabilityChangedEvent {
    pub id: RetailerOfferId,
    pub previous_availability: Option<Availability>,
    pub availability: Option<Availability>,
    pub updated_at: DateTime<Utc>,
}

impl OfferAvailabilityChangedEvent {
    pub fn new(
        id: RetailerOfferId,
        previous_availability: Option<Availability>,
        availability: Option<Availability>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            previous_availability,
            availability,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferAvailabilityChangedEvent {
    fn event_type(&self) -> &'static str {
        "OfferAvailabilityChangedNextGeneration"
    }
}
