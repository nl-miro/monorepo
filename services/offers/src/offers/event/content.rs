use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;

use crate::offers::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferDescriptionChangedEvent {
    pub id: RetailerOfferId,
    pub previous_description: String,
    pub description: String,
    pub updated_at: DateTime<Utc>,
}

impl OfferDescriptionChangedEvent {
    pub fn new(
        id: RetailerOfferId,
        previous_description: String,
        description: String,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            previous_description,
            description,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferDescriptionChangedEvent {
    fn event_type(&self) -> &'static str {
        "OfferDescriptionChanged"
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferImageRemovedEvent {
    pub id: RetailerOfferId,
    pub image_url: String,
    pub updated_at: DateTime<Utc>,
}

impl OfferImageRemovedEvent {
    pub fn new(id: RetailerOfferId, image_url: String, updated_at: DateTime<Utc>) -> Self {
        Self {
            id,
            image_url,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferImageRemovedEvent {
    fn event_type(&self) -> &'static str {
        "OfferImageRemoved"
    }
}
