use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;

use crate::offers::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferLanguageChangedEvent {
    pub id: RetailerOfferId,
    pub previous_language: String,
    pub language: String,
    pub updated_at: DateTime<Utc>,
}

impl OfferLanguageChangedEvent {
    pub fn new(
        id: RetailerOfferId,
        previous_language: String,
        language: String,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            previous_language,
            language,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferLanguageChangedEvent {
    fn event_type(&self) -> &'static str {
        "OfferLanguageChanged"
    }
}
