use bodul_shared::retailer::RetailerCode;
use money::Money;
use retailer_sauce::availability::Availability;
use retailer_sauce::identifier::{
    Gtin, OfferIdentifiers, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku,
};
use retailer_sauce::item_condition::ItemCondition;
use retailer_sauce::localization::{Localization, TargetMarket};
use serde::{Deserialize, Serialize};
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

/// Payload prepared for offer import.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferImport {
    pub retailer_code: RetailerCode,
    pub url: String,
    pub content: ContentNextGeneration,
    pub price: Money,
    pub was_price: Option<Money>,
    pub availability: Option<Availability>,
    pub item_condition: Option<ItemCondition>,
    pub images: Vec<String>,
    pub identifiers: RetailerIdentifiers,
    pub target_market: TargetMarket,
    pub language: String,
    pub scraped_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentNextGeneration {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetailerIdentifiers {
    pub gtin: Option<Gtin>,
    pub sku: Option<Sku>,
    pub shopify_product_gid: Option<ShopifyProductGid>,
    pub shopify_product_variation_gid: Option<ShopifyProductVariationGid>,
    pub shopify_handle: Option<ShopifyHandle>,
}

impl From<OfferIdentifiers> for RetailerIdentifiers {
    fn from(value: OfferIdentifiers) -> Self {
        Self {
            gtin: value.gtin,
            sku: value.sku,
            shopify_product_gid: value.shopify_product_gid,
            shopify_product_variation_gid: value.shopify_product_variation_gid,
            shopify_handle: value.shopify_handle,
        }
    }
}

impl RetailerIdentifiers {
    pub(super) fn guard(&self) -> Result<(), String> {
        let identifiers = OfferIdentifiers {
            gtin: self.gtin.clone(),
            sku: self.sku.clone(),
            shopify_product_gid: self.shopify_product_gid.clone(),
            shopify_product_variation_gid: self.shopify_product_variation_gid.clone(),
            shopify_handle: self.shopify_handle.clone(),
        };
        identifiers.guard()
    }
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
            if payload.was_price.as_ref().is_some_and(|was| {
                was.currency() != payload.price.currency() || was.amount() <= payload.price.amount()
            }) || payload
                .availability
                .as_ref()
                .is_some_and(|v| v.guard().is_err())
                || payload
                    .item_condition
                    .as_ref()
                    .is_some_and(|v| v.guard().is_err())
                || payload
                    .content
                    .description
                    .as_ref()
                    .is_some_and(|value| value.contains('\0'))
            {
                return false;
            }

            if entry.payload.identifiers.guard().is_err() {
                return false;
            }
            let localization = Localization {
                target_market: Some(entry.payload.target_market.clone()),
                language: Some(entry.payload.language.clone()),
            };
            if localization.guard().is_err()
                || Some(entry.payload.scraped_at) != self.source.storage.scraped_at
            {
                return false;
            }
            let images = &entry.payload.images;
            let (filtered, skipped) = shared::images::filter(images.clone());
            skipped == 0 && &filtered == images
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
    use super::*;
    #[test]
    fn import_groups_all_identifiers_and_flattens_market_and_language() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/offer_export/import_with_identifiers.json"
        ))
        .unwrap();
        let mut wire = fixture.clone();
        for field in ["was_price", "availability", "item_condition"] {
            wire[field] = serde_json::Value::Null;
        }
        let payload: OfferImport = serde_json::from_value(wire.clone()).unwrap();
        assert!(payload.identifiers.guard().is_ok());
        let encoded = serde_json::to_value(payload).unwrap();
        for field in ["gtin", "sku"] {
            assert_eq!(encoded["identifiers"][field], fixture["identifiers"][field]);
            assert!(encoded.get(field).is_none());
        }
        for field in [
            "shopify_product_gid",
            "shopify_product_variation_gid",
            "shopify_handle",
        ] {
            assert_eq!(encoded["identifiers"][field], fixture["identifiers"][field]);
        }
        for field in ["target_market", "language"] {
            assert_eq!(encoded[field], fixture[field]);
        }
        assert!(encoded.get("external_identifiers").is_none());
        assert!(encoded.get("localization").is_none());
        assert_eq!(encoded, wire);
    }

    #[test]
    fn optional_facts_normalize_missing_and_null() {
        let original: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/offer_export/import_with_facts.json"
        ))
        .unwrap();
        for field in ["was_price", "availability", "item_condition"] {
            let mut missing = original.clone();
            missing.as_object_mut().unwrap().remove(field);
            let mut null = original.clone();
            null[field] = serde_json::Value::Null;
            let missing: OfferImport = serde_json::from_value(missing).unwrap();
            let null: OfferImport = serde_json::from_value(null).unwrap();
            assert_eq!(missing, null);
            assert_eq!(
                serde_json::to_value(missing).unwrap()[field],
                serde_json::Value::Null
            );
        }
    }

    #[test]
    fn required_export_fields_reject_missing_and_null() {
        let payload: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/offer_export/import_with_identifiers.json"
        ))
        .unwrap();
        assert!(serde_json::from_value::<OfferImport>(payload.clone()).is_ok());
        for field in [
            "scraped_at",
            "identifiers",
            "content",
            "target_market",
            "language",
        ] {
            let mut missing = payload.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<OfferImport>(missing).is_err());
            let mut null = payload.clone();
            null[field] = serde_json::Value::Null;
            assert!(serde_json::from_value::<OfferImport>(null).is_err());
        }
    }

    #[test]
    fn payload_images_require_an_array_and_serialize_empty_arrays() {
        let value: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/offer_export/import_payload.json"
        ))
        .unwrap();
        let mut value = value;
        value.as_object_mut().unwrap().remove("images");
        value["target_market"] = serde_json::json!({"kind":"country","code":"AU"});
        value["language"] = serde_json::json!("en");
        value["scraped_at"] = serde_json::json!("2026-10-26T12:00:00.123456Z");
        assert!(serde_json::from_value::<OfferImport>(value.clone()).is_err());
        value["images"] = serde_json::json!([]);
        let payload: OfferImport = serde_json::from_value(value.clone()).unwrap();
        assert!(payload.images.is_empty());
        assert_eq!(
            serde_json::to_value(payload).unwrap()["images"],
            serde_json::json!([])
        );
        value["images"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<OfferImport>(value).is_err());
    }
}
