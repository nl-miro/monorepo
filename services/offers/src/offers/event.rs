use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;
use money::Money;
use testimony::prelude::HasAggregateId;

use super::retailer_offer_id::RetailerOfferId;

mod availability;
pub use availability::OfferAvailabilityChangedEvent;

mod item_condition;
pub use item_condition::OfferItemConditionChangedEvent;

mod created;
pub use created::OfferCreatedEvent;

mod pricing;
pub use pricing::{OfferHistoricPriceAddedEvent, OfferPriceChangedEvent, OfferPriceConfirmedEvent};

mod content;
pub use content::{OfferDescriptionChangedEvent, OfferImageRemovedEvent};

mod target_market;
pub use target_market::OfferTargetMarketChangedEvent;

mod language;
pub use language::OfferLanguageChangedEvent;

mod identifiers;
pub use identifiers::{
    OfferGtinChangedEvent, OfferShopifyProductGidChangedEvent,
    OfferShopifyProductVariantGidChangedEvent, OfferSkuChangedEvent,
};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)] // Keep next-generation creation inline.
pub enum RetailerOfferEvent {
    OfferAvailabilityChanged(OfferAvailabilityChangedEvent),
    OfferItemConditionChanged(OfferItemConditionChangedEvent),

    OfferPriceChanged(OfferPriceChangedEvent),
    OfferPriceConfirmed(OfferPriceConfirmedEvent),
    OfferHistoricPriceAdded(OfferHistoricPriceAddedEvent),
    OfferDescriptionChanged(OfferDescriptionChangedEvent),
    OfferImageRemoved(OfferImageRemovedEvent),
    OfferLanguageChanged(OfferLanguageChangedEvent),
    OfferTargetMarketChanged(OfferTargetMarketChangedEvent),
    OfferGtinChanged(OfferGtinChangedEvent),
    OfferSkuChanged(OfferSkuChangedEvent),
    OfferShopifyProductGidChanged(OfferShopifyProductGidChangedEvent),
    OfferShopifyProductVariantGidChanged(OfferShopifyProductVariantGidChangedEvent),
    OfferCreated(OfferCreatedEvent),
    OfferImageAdded(OfferImageAddedEvent),
    OfferTitleChanged(OfferTitleChangedEvent),
}

impl ApplicationEvent for RetailerOfferEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::OfferAvailabilityChanged(event) => event.event_type(),
            Self::OfferItemConditionChanged(event) => event.event_type(),

            Self::OfferPriceChanged(event) => event.event_type(),
            Self::OfferPriceConfirmed(event) => event.event_type(),
            Self::OfferHistoricPriceAdded(event) => event.event_type(),
            Self::OfferDescriptionChanged(event) => event.event_type(),
            Self::OfferImageRemoved(event) => event.event_type(),
            Self::OfferLanguageChanged(event) => event.event_type(),
            Self::OfferTargetMarketChanged(event) => event.event_type(),
            Self::OfferGtinChanged(event) => event.event_type(),
            Self::OfferSkuChanged(event) => event.event_type(),
            Self::OfferShopifyProductGidChanged(event) => event.event_type(),
            Self::OfferShopifyProductVariantGidChanged(event) => event.event_type(),
            Self::OfferCreated(event) => event.event_type(),
            Self::OfferImageAdded(event) => event.event_type(),
            Self::OfferTitleChanged(event) => event.event_type(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(from = "OfferCreatedWire")]
pub struct HistoricalOfferCreatedEvent {
    #[serde(flatten)]
    pub facts: OfferFacts,
    #[serde(flatten)]
    pub identifiers: retailer_sauce::identifier::OfferIdentifiers,
    #[serde(default)]
    pub localization: retailer_sauce::localization::Localization,
    #[serde(default)]
    pub scraped_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub images: Vec<String>,
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum OfferCreatedWire {
    Current(CurrentOfferCreatedEvent),
    Legacy(LegacyOfferCreatedEvent),
}

#[derive(serde::Deserialize)]
struct CurrentOfferCreatedEvent {
    #[serde(flatten)]
    pub facts: OfferFacts,
    #[serde(flatten)]
    pub identifiers: retailer_sauce::identifier::OfferIdentifiers,
    #[serde(default)]
    pub localization: retailer_sauce::localization::Localization,
    #[serde(default)]
    pub scraped_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub images: Vec<String>,
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

/// Explicit historical DTO: versions 1–8 carried a singular description object.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LegacyOfferCreatedEvent {
    #[serde(flatten)]
    pub facts: LegacyOfferFacts,
    #[serde(flatten)]
    pub identifiers: retailer_sauce::identifier::OfferIdentifiers,
    #[serde(default)]
    pub localization: retailer_sauce::localization::Localization,
    #[serde(default)]
    pub scraped_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub images: Vec<String>,
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LegacyOfferFacts {
    #[serde(default, deserialize_with = "retailer_sauce::json::nullable_object")]
    pub was_price: Option<Money>,
    #[serde(default)]
    pub availability: Option<retailer_sauce::availability::Availability>,
    #[serde(default)]
    pub item_condition: Option<retailer_sauce::item_condition::ItemCondition>,
    #[serde(default)]
    pub content: retailer_sauce::structuralized::content::Content,
}
impl From<OfferCreatedWire> for HistoricalOfferCreatedEvent {
    fn from(value: OfferCreatedWire) -> Self {
        match value {
            OfferCreatedWire::Current(value) => value.into(),
            OfferCreatedWire::Legacy(value) => value.into(),
        }
    }
}
impl From<CurrentOfferCreatedEvent> for HistoricalOfferCreatedEvent {
    fn from(mut value: CurrentOfferCreatedEvent) -> Self {
        retailer_sauce::offer_content::sort(&mut value.facts.content);
        Self {
            facts: value.facts,
            identifiers: value.identifiers,
            localization: value.localization,
            scraped_at: value.scraped_at,
            images: value.images,
            id: value.id,
            retailer_code: value.retailer_code,
            title: value.title,
            price: value.price,
            url: value.url,
            created_at: value.created_at,
        }
    }
}
impl From<LegacyOfferCreatedEvent> for HistoricalOfferCreatedEvent {
    fn from(value: LegacyOfferCreatedEvent) -> Self {
        let content = retailer_sauce::offer_content::singleton(
            value.title.clone(),
            value.localization.language.clone(),
            value.facts.content.description,
        );
        let facts = OfferFacts {
            was_price: value.facts.was_price,
            availability: value.facts.availability,
            item_condition: value.facts.item_condition,
            content,
        };
        Self {
            facts,
            identifiers: value.identifiers,
            localization: value.localization,
            scraped_at: value.scraped_at,
            images: value.images,
            id: value.id,
            retailer_code: value.retailer_code,
            title: value.title,
            price: value.price,
            url: value.url,
            created_at: value.created_at,
        }
    }
}
impl HistoricalOfferCreatedEvent {
    pub fn guard(&self) -> Result<(), String> {
        self.facts.guard()?;
        self.localization.guard()?;
        self.identifiers.guard()?;
        retailer_sauce::offer_content::guard(&self.facts.content, &self.localization.language)?;
        if retailer_sauce::offer_content::selected(
            &self.facts.content,
            &self.localization.language,
        )?
        .title
            != self.title
        {
            return Err(err_selected_title());
        }
        guard_images(&self.images)
    }
}
fn err_selected_title() -> String {
    "title must match selected content".into()
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct OfferTitleChangedEvent {
    pub id: RetailerOfferId,
    pub title: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoricalOfferPriceChangedEvent {
    pub id: RetailerOfferId,
    pub price: Money,
}

impl HasAggregateId for RetailerOfferEvent {
    fn aggregate_id(&self) -> String {
        match self {
            Self::OfferAvailabilityChanged(event) => event.id.to_string(),
            Self::OfferItemConditionChanged(event) => event.id.to_string(),

            Self::OfferPriceChanged(event) => event.id.to_string(),
            Self::OfferPriceConfirmed(event) => event.id.to_string(),
            Self::OfferHistoricPriceAdded(event) => event.id.to_string(),
            Self::OfferDescriptionChanged(event) => event.id.to_string(),
            Self::OfferImageRemoved(event) => event.id.to_string(),
            Self::OfferLanguageChanged(event) => event.id.to_string(),
            Self::OfferTargetMarketChanged(event) => event.id.to_string(),
            Self::OfferGtinChanged(event) => event.id.to_string(),
            Self::OfferSkuChanged(event) => event.id.to_string(),
            Self::OfferShopifyProductGidChanged(event) => event.id.to_string(),
            Self::OfferShopifyProductVariantGidChanged(event) => event.id.to_string(),
            Self::OfferCreated(event) => event.id.to_string(),
            Self::OfferImageAdded(event) => event.id.to_string(),
            Self::OfferTitleChanged(event) => event.id.to_string(),
        }
    }
}

pub const OFFER_CREATED_EVENT: &str = "OfferCreated";
pub const OFFER_TITLE_CHANGED_EVENT: &str = "OfferTitleChanged";
pub const OFFER_PRICE_CHANGED_EVENT: &str = "OfferPriceChangedEvent";

impl ApplicationEvent for HistoricalOfferCreatedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_CREATED_EVENT
    }
}
impl OfferTitleChangedEvent {
    pub fn new(id: RetailerOfferId, title: String) -> Self {
        Self { id, title }
    }
}

impl ApplicationEvent for OfferTitleChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_TITLE_CHANGED_EVENT
    }
}
impl ApplicationEvent for HistoricalOfferPriceChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_PRICE_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferImageAddedEvent {
    pub id: RetailerOfferId,
    pub image_url: String,
    pub position: u32,
}
pub const OFFER_IMAGE_ADDED_EVENT: &str = "OfferImageAdded";
impl OfferImageAddedEvent {
    pub fn new(id: RetailerOfferId, image_url: String, position: u32) -> Self {
        Self {
            id,
            image_url,
            position,
        }
    }
}

impl ApplicationEvent for OfferImageAddedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_IMAGE_ADDED_EVENT
    }
}

/// Events contain validated, ordered unique images; filtering happens at import.
pub(crate) fn guard_images(images: &[String]) -> Result<(), String> {
    if images.len() as u64 > u64::from(u32::MAX) + 1 {
        return Err(err_invalid_images());
    }
    let (filtered, skipped) = shared::images::filter(images.to_vec());
    if skipped != 0 || filtered != images {
        return Err(err_invalid_images());
    }
    Ok(())
}

fn err_invalid_images() -> String {
    "creation images must be valid, unique URLs within position capacity".into()
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferScrapedAtUpdatedEvent {
    pub id: RetailerOfferId,
    pub scraped_at: DateTime<Utc>,
}
pub const OFFER_SCRAPED_AT_UPDATED_EVENT: &str = "OfferScrapedAtUpdated";
impl ApplicationEvent for OfferScrapedAtUpdatedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_SCRAPED_AT_UPDATED_EVENT
    }
}

/// Preserve the exact Testimony envelope revision without changing the payload.
#[derive(serde::Serialize)]
#[serde(transparent)]
pub struct SequencedOfferEvent {
    pub event: RetailerOfferEvent,
    #[serde(skip)]
    pub sequence: u64,
}
impl ApplicationEvent for SequencedOfferEvent {
    fn event_type(&self) -> &'static str {
        self.event.event_type()
    }
    fn sequence(&self) -> Option<u64> {
        Some(self.sequence)
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferLocalizationChangedEvent {
    pub id: RetailerOfferId,
    pub localization: retailer_sauce::localization::Localization,
}
pub const OFFER_LOCALIZATION_CHANGED_EVENT: &str = "OfferLocalizationChanged";
impl ApplicationEvent for OfferLocalizationChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_LOCALIZATION_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferIdentifiersChangedEvent {
    pub id: RetailerOfferId,
    #[serde(
        flatten,
        deserialize_with = "retailer_sauce::identifier::complete_identifiers"
    )]
    pub identifiers: retailer_sauce::identifier::OfferIdentifiers,
}
pub const OFFER_IDENTIFIERS_CHANGED_EVENT: &str = "OfferIdentifiersChanged";
impl ApplicationEvent for OfferIdentifiersChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_IDENTIFIERS_CHANGED_EVENT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_do_not_capture_other_untagged_events_or_accept_partial_sets() {
        let id = "00000000-0000-0000-0000-000000000001";
        for (fields, kind) in [
            (
                serde_json::json!({"title":"changed"}),
                OFFER_TITLE_CHANGED_EVENT,
            ),
            (
                serde_json::json!({"localization":{"target_market":null,"language":null}}),
                OFFER_LOCALIZATION_CHANGED_EVENT,
            ),
            (
                serde_json::json!({"scraped_at":"2026-10-26T12:00:00Z"}),
                OFFER_SCRAPED_AT_UPDATED_EVENT,
            ),
            (
                serde_json::json!({"gtin":null,"sku":null,"external_identifiers":{}}),
                OFFER_IDENTIFIERS_CHANGED_EVENT,
            ),
        ] {
            let mut payload = fields;
            payload["id"] = serde_json::json!(id);
            if kind == OFFER_TITLE_CHANGED_EVENT {
                let event: RetailerOfferEvent = serde_json::from_value(payload).unwrap();
                assert_eq!(event.event_type(), kind);
            } else {
                assert!(serde_json::from_value::<RetailerOfferEvent>(payload).is_err());
            }
        }
        assert!(
            serde_json::from_value::<OfferIdentifiersChangedEvent>(
                serde_json::json!({"id":id,"sku":"partial"})
            )
            .is_err()
        );
    }
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferFacts {
    #[serde(default, deserialize_with = "retailer_sauce::json::nullable_object")]
    pub was_price: Option<Money>,
    #[serde(default)]
    pub availability: Option<retailer_sauce::availability::Availability>,
    #[serde(default)]
    pub item_condition: Option<retailer_sauce::item_condition::ItemCondition>,
    pub content: Vec<retailer_sauce::offer_content::Content>,
}
impl OfferFacts {
    pub fn guard(&self) -> Result<(), String> {
        if let Some(value) = &self.availability {
            value.guard()?;
        }
        if let Some(value) = &self.item_condition {
            value.guard()?;
        }
        for content in &self.content {
            content.guard()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Pricing {
    #[serde(deserialize_with = "retailer_sauce::json::object")]
    pub price: Money,
    #[serde(deserialize_with = "retailer_sauce::json::nullable_object")]
    pub was_price: Option<Money>,
}
impl Pricing {
    pub fn guard(&self) -> Result<(), String> {
        guard_was_price(&self.price, self.was_price.as_ref())
    }
}

pub fn guard_was_price(price: &Money, was_price: Option<&Money>) -> Result<(), String> {
    if was_price
        .is_some_and(|was| was.currency() != price.currency() || was.amount() <= price.amount())
    {
        return Err(err_invalid_was_price());
    }
    Ok(())
}
fn err_invalid_was_price() -> String {
    "was_price must use the current currency and be greater than price".into()
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferPricingChangedEvent {
    pub id: RetailerOfferId,
    #[serde(deserialize_with = "retailer_sauce::json::object")]
    pub pricing: Pricing,
}
pub const OFFER_PRICING_CHANGED_EVENT: &str = "OfferPricingChanged";
impl ApplicationEvent for OfferPricingChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_PRICING_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HistoricalOfferAvailabilityChangedEvent {
    pub id: RetailerOfferId,
    #[serde(deserialize_with = "retailer_sauce::json::required_nullable")]
    pub availability: Option<retailer_sauce::availability::Availability>,
}
pub const OFFER_AVAILABILITY_CHANGED_EVENT: &str = "OfferAvailabilityChanged";
impl ApplicationEvent for HistoricalOfferAvailabilityChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_AVAILABILITY_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HistoricalOfferItemConditionChangedEvent {
    pub id: RetailerOfferId,
    #[serde(deserialize_with = "retailer_sauce::json::required_nullable")]
    pub item_condition: Option<retailer_sauce::item_condition::ItemCondition>,
}
pub const OFFER_ITEM_CONDITION_CHANGED_EVENT: &str = "OfferItemConditionChanged";
impl ApplicationEvent for HistoricalOfferItemConditionChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_ITEM_CONDITION_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferContentChangedEvent {
    pub id: RetailerOfferId,
    pub content: retailer_sauce::structuralized::content::Content,
}
pub const OFFER_CONTENT_CHANGED_EVENT: &str = "OfferContentChanged";
impl ApplicationEvent for OfferContentChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_CONTENT_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferContentsChangedEvent {
    pub id: RetailerOfferId,
    pub content: Vec<retailer_sauce::offer_content::Content>,
    #[serde(deserialize_with = "retailer_sauce::json::required_nullable")]
    pub language: Option<String>,
}
pub const OFFER_CONTENTS_CHANGED_EVENT: &str = "OfferContentsChanged";
impl ApplicationEvent for OfferContentsChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_CONTENTS_CHANGED_EVENT
    }
}
impl OfferContentsChangedEvent {
    pub fn guard(&self) -> Result<(), String> {
        retailer_sauce::offer_content::guard(&self.content, &self.language)
    }
}
