use crate::offers::event::*;
use serde::{Deserialize, Serialize};
use testimony::prelude::HasAggregateId;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[allow(clippy::large_enum_variant)]
pub enum OfferEventRecord {
    #[serde(rename = "OfferAvailabilityChangedNextGeneration.v1")]
    AvailabilityChanged(OfferAvailabilityChangedEvent),
    #[serde(rename = "OfferItemConditionChangedNextGeneration.v1")]
    ItemConditionChanged(OfferItemConditionChangedEvent),
    #[serde(rename = "OfferPriceChangedNextGeneration.v1")]
    PriceChanged(OfferPriceChangedEvent),
    #[serde(rename = "OfferPriceConfirmed.v1")]
    PriceConfirmed(OfferPriceConfirmedEvent),
    #[serde(rename = "OfferHistoricPriceAdded.v1")]
    HistoricPriceAdded(OfferHistoricPriceAddedEvent),
    #[serde(rename = "OfferDescriptionChanged.v1")]
    DescriptionChanged(OfferDescriptionChangedEvent),
    #[serde(rename = "OfferImageRemoved.v1")]
    ImageRemoved(OfferImageRemovedEvent),
    #[serde(rename = "OfferLanguageChanged.v1")]
    LanguageChanged(OfferLanguageChangedEvent),
    #[serde(rename = "OfferTargetMarketChanged.v1")]
    TargetMarketChanged(OfferTargetMarketChangedEvent),
    #[serde(rename = "OfferGtinChanged.v1")]
    GtinChanged(OfferGtinChangedEvent),
    #[serde(rename = "OfferSkuChanged.v1")]
    SkuChanged(OfferSkuChangedEvent),
    #[serde(rename = "OfferShopifyProductGidChanged.v1")]
    ShopifyProductGidChanged(OfferShopifyProductGidChangedEvent),
    #[serde(rename = "OfferShopifyProductVariantGidChanged.v1")]
    ShopifyProductVariantGidChanged(OfferShopifyProductVariantGidChangedEvent),
    #[serde(rename = "OfferCreatedNextGeneration.v1")]
    Created(OfferCreatedEvent),
    #[serde(rename = "OfferImageAdded.v1")]
    ImageAdded(OfferImageAddedEvent),
    #[serde(rename = "OfferTitleChanged.v1")]
    TitleChanged(OfferTitleChangedEvent),
}

impl From<&RetailerOfferEvent> for OfferEventRecord {
    fn from(event: &RetailerOfferEvent) -> Self {
        match event {
            RetailerOfferEvent::OfferAvailabilityChanged(event) => {
                Self::AvailabilityChanged(event.clone())
            }
            RetailerOfferEvent::OfferItemConditionChanged(event) => {
                Self::ItemConditionChanged(event.clone())
            }
            RetailerOfferEvent::OfferPriceChanged(event) => Self::PriceChanged(event.clone()),
            RetailerOfferEvent::OfferPriceConfirmed(event) => Self::PriceConfirmed(event.clone()),
            RetailerOfferEvent::OfferHistoricPriceAdded(event) => {
                Self::HistoricPriceAdded(event.clone())
            }
            RetailerOfferEvent::OfferDescriptionChanged(event) => {
                Self::DescriptionChanged(event.clone())
            }
            RetailerOfferEvent::OfferImageRemoved(event) => Self::ImageRemoved(event.clone()),
            RetailerOfferEvent::OfferLanguageChanged(event) => Self::LanguageChanged(event.clone()),
            RetailerOfferEvent::OfferTargetMarketChanged(event) => {
                Self::TargetMarketChanged(event.clone())
            }
            RetailerOfferEvent::OfferGtinChanged(event) => Self::GtinChanged(event.clone()),
            RetailerOfferEvent::OfferSkuChanged(event) => Self::SkuChanged(event.clone()),
            RetailerOfferEvent::OfferShopifyProductGidChanged(event) => {
                Self::ShopifyProductGidChanged(event.clone())
            }
            RetailerOfferEvent::OfferShopifyProductVariantGidChanged(event) => {
                Self::ShopifyProductVariantGidChanged(event.clone())
            }
            RetailerOfferEvent::OfferCreated(event) => Self::Created(event.clone()),
            RetailerOfferEvent::OfferImageAdded(event) => Self::ImageAdded(event.clone()),
            RetailerOfferEvent::OfferTitleChanged(event) => Self::TitleChanged(event.clone()),
        }
    }
}

impl From<OfferEventRecord> for RetailerOfferEvent {
    fn from(record: OfferEventRecord) -> Self {
        match record {
            OfferEventRecord::AvailabilityChanged(event) => Self::OfferAvailabilityChanged(event),
            OfferEventRecord::ItemConditionChanged(event) => Self::OfferItemConditionChanged(event),
            OfferEventRecord::PriceChanged(event) => Self::OfferPriceChanged(event),
            OfferEventRecord::PriceConfirmed(event) => Self::OfferPriceConfirmed(event),
            OfferEventRecord::HistoricPriceAdded(event) => Self::OfferHistoricPriceAdded(event),
            OfferEventRecord::DescriptionChanged(event) => Self::OfferDescriptionChanged(event),
            OfferEventRecord::ImageRemoved(event) => Self::OfferImageRemoved(event),
            OfferEventRecord::LanguageChanged(event) => Self::OfferLanguageChanged(event),
            OfferEventRecord::TargetMarketChanged(event) => Self::OfferTargetMarketChanged(event),
            OfferEventRecord::GtinChanged(event) => Self::OfferGtinChanged(event),
            OfferEventRecord::SkuChanged(event) => Self::OfferSkuChanged(event),
            OfferEventRecord::ShopifyProductGidChanged(event) => {
                Self::OfferShopifyProductGidChanged(event)
            }
            OfferEventRecord::ShopifyProductVariantGidChanged(event) => {
                Self::OfferShopifyProductVariantGidChanged(event)
            }
            OfferEventRecord::Created(event) => Self::OfferCreated(event),
            OfferEventRecord::ImageAdded(event) => Self::OfferImageAdded(event),
            OfferEventRecord::TitleChanged(event) => Self::OfferTitleChanged(event),
        }
    }
}

impl HasAggregateId for OfferEventRecord {
    fn aggregate_id(&self) -> String {
        match self {
            Self::AvailabilityChanged(event) => event.id.to_string(),
            Self::ItemConditionChanged(event) => event.id.to_string(),
            Self::PriceChanged(event) => event.id.to_string(),
            Self::PriceConfirmed(event) => event.id.to_string(),
            Self::HistoricPriceAdded(event) => event.id.to_string(),
            Self::DescriptionChanged(event) => event.id.to_string(),
            Self::ImageRemoved(event) => event.id.to_string(),
            Self::LanguageChanged(event) => event.id.to_string(),
            Self::TargetMarketChanged(event) => event.id.to_string(),
            Self::GtinChanged(event) => event.id.to_string(),
            Self::SkuChanged(event) => event.id.to_string(),
            Self::ShopifyProductGidChanged(event) => event.id.to_string(),
            Self::ShopifyProductVariantGidChanged(event) => event.id.to_string(),
            Self::Created(event) => event.id.to_string(),
            Self::ImageAdded(event) => event.id.to_string(),
            Self::TitleChanged(event) => event.id.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_record_tags_are_rejected() {
        for tag in [
            "OfferCreated.v10",
            "OfferCreated.v1",
            "OfferPricingChanged.v1",
            "OfferAvailabilityChanged.v1",
            "OfferItemConditionChanged.v1",
            "OfferContentChanged.v1",
            "OfferContentsChanged.v1",
            "OfferIdentifiersChanged.v1",
            "OfferLocalizationChanged.v2",
            "OfferPriceChanged.v1",
            "OfferScrapedAtUpdated.v1",
        ] {
            let payload = serde_json::json!({"type":tag,"data":{}});
            let error = serde_json::from_value::<OfferEventRecord>(payload)
                .err()
                .unwrap();
            assert!(
                error.to_string().contains("unknown variant"),
                "{tag}: {error}"
            );
        }
    }
}
