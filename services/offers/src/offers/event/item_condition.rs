use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;
use retailer_sauce::item_condition::ItemCondition;

use crate::offers::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferItemConditionChangedEvent {
    pub id: RetailerOfferId,
    pub previous_item_condition: Option<ItemCondition>,
    pub item_condition: Option<ItemCondition>,
    pub updated_at: DateTime<Utc>,
}

impl OfferItemConditionChangedEvent {
    pub fn new(
        id: RetailerOfferId,
        previous_item_condition: Option<ItemCondition>,
        item_condition: Option<ItemCondition>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            previous_item_condition,
            item_condition,
            updated_at,
        }
    }
}

impl ApplicationEvent for OfferItemConditionChangedEvent {
    fn event_type(&self) -> &'static str {
        "OfferItemConditionChangedNextGeneration"
    }
}
