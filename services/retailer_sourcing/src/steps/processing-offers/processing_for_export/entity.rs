use chrono::DateTime;
use chrono::Utc;
use retailer_sauce::prelude::Availability;
use retailer_sauce::prelude::ExternalIdentifiers;
use retailer_sauce::prelude::Gtin;
use retailer_sauce::prelude::ItemCondition;
use retailer_sauce::prelude::Localization;
use retailer_sauce::prelude::ProductIdentifiers;
use retailer_sauce::prelude::Sku;
use retailer_sauce::prelude::StructuralizedContent;
use retailer_sauce::prelude::TargetMarket;
use serde::Deserializer;
use shared::images::filter;

use money::Money;
use retailer_sauce::prelude::Observation;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

use crate::platform::object_storage::{ObjectStorageMetadata, OfferStorageMetadata};

/// The complete immutable input, independent of the latest pipeline metadata row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportSource {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
    pub storage: OfferStorageMetadata,
}

/// Matches offers' ImportOfferDetails command; this is not its HTTP form DTO.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferImport {
    #[serde(
        default,
        skip_serializing_if = "Observation::is_missing",
        deserialize_with = "retailer_sauce::prelude::object_observation"
    )]
    pub was_price: Observation<Money>,
    #[serde(default, skip_serializing_if = "Observation::is_missing")]
    pub availability: Observation<Availability>,
    #[serde(default, skip_serializing_if = "Observation::is_missing")]
    pub item_condition: Observation<ItemCondition>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "retailer_sauce::prelude::structuralized::observation"
    )]
    pub content: Option<StructuralizedContent>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gtin: Option<Gtin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sku: Option<Sku>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "retailer_sauce::prelude::external_object"
    )]
    pub external_identifiers: Option<ExternalIdentifiers>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "retailer_sauce::prelude::localization::observation"
    )]
    pub localization: Option<Localization>,
    pub acquired_at: DateTime<Utc>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "image_array"
    )]
    pub images: Option<Vec<String>>,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
}

fn image_array<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error> {
    Vec::<String>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportEntry {
    /// Delivery identity, not the receiving service's offer identity.
    pub id: Uuid,
    pub variation_index: usize,
    pub payload: OfferImport,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionReason {
    MissingPrice,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportExclusion {
    pub variation_index: usize,
    pub url: String,
    pub reason: ExclusionReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferExport {
    pub id: Uuid,
    pub source: ExportSource,
    pub entries: Vec<ExportEntry>,
    pub exclusions: Vec<ExportExclusion>,
}

impl OfferExport {
    pub fn valid_schema(&self) -> bool {
        self.entries.iter().all(|entry| {
            let payload = &entry.payload;
            let market = payload
                .localization
                .as_ref()
                .and_then(|value| value.target_market.as_ref());
            if matches!(market, Some(TargetMarket::LegacyRegion { .. }))
                || payload.was_price.value().is_some_and(|was| {
                    was.currency() != payload.price.currency()
                        || was.amount() <= payload.price.amount()
                })
                || payload
                    .availability
                    .value()
                    .is_some_and(|v| v.guard().is_err())
                || payload
                    .item_condition
                    .value()
                    .is_some_and(|v| v.guard().is_err())
                || payload.content.as_ref().is_some_and(|v| v.guard().is_err())
            {
                return false;
            }

            let Some(external_identifiers) = payload.external_identifiers.clone() else {
                return false;
            };
            let identifiers = ProductIdentifiers {
                gtin: payload.gtin.clone(),
                sku: payload.sku.clone(),
                external_identifiers,
            };
            if identifiers.guard().is_err() {
                return false;
            }
            let Some(localization) = &payload.localization else {
                return false;
            };
            if localization.guard().is_err() {
                return false;
            }
            if payload.acquired_at != self.source.storage.acquired_at {
                return false;
            }
            payload.images.as_ref().is_some_and(|images| {
                let (filtered, skipped) = filter(images.clone());
                skipped == 0 && &filtered == images
            })
        })
    }
}

/// Pending bytes survive crashes; completed snapshots are loaded from object storage.
#[derive(Debug, Clone)]
pub(super) struct PreparedExport {
    pub snapshot: OfferExport,
    pub body: String,
    pub storage: Option<ObjectStorageMetadata>,
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use serde_json::from_str;
    use serde_json::from_value;
    use serde_json::json as serde_json_json;

    use super::*;
    #[test]
    fn payload_images_distinguish_omission_from_malformed_null() {
        let mut value: Value = from_str(include_str!(
            "../../../../tests/fixtures/offer_export/import_payload.json"
        ))
        .unwrap();
        value.as_object_mut().unwrap().remove("images");
        assert!(
            from_value::<OfferImport>(value.clone())
                .unwrap()
                .images
                .is_none()
        );
        value["images"] = serde_json_json!([]);
        assert_eq!(
            from_value::<OfferImport>(value.clone()).unwrap().images,
            Some(vec![])
        );
        value["images"] = Value::Null;
        assert!(from_value::<OfferImport>(value).is_err());
    }
}
