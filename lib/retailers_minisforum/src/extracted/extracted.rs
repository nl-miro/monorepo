//! Extracted Minisforum offer-details model converted from `graphql::raw`.
//! Converted from raw via `raw_to_extracted`; reshape here, not in raw.
//! Monetary values use currency-aware integer minor units via `money::Money`;
//! other numeric scalars retain their JSON representation. See fixtures/README.md.

pub mod collection;
pub mod graphql;
pub mod image;
pub mod media;
pub mod not_interested;
pub mod prelude;
pub mod product;

use collection::Collection;
use media::Media;
use product::Product;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Extracted product data without raw artifact metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(try_from = "ArtifactWire")]
pub struct ExtractedOfferDetails {
    // pub schema_version: u32,
    pub acquired_at: DateTime<Utc>,
    pub context: retailer_sauce::shopify::markets::MarketContext,
    pub product: Product,
    pub collections: Vec<Collection>,
    pub images: Vec<Image>,
    pub media: Vec<Media>,
    // Original request/response evidence belongs to the raw artifact.
    // pub pages: Vec<RecordedPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seo {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub description: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    // #[serde(rename = "__typename")]
    // pub type_name: String,
    pub id: Option<String>,
    pub thumbhash: Option<String>,
    pub url: String,
    pub alt_text: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

#[cfg(test)]
mod contract_tests {
    use super::prelude::*;
    use crate::graphql::raw;
    use serde_json::{Value, json};
    #[test]
    fn nested_fields_and_money_are_strict() {
        let product: Value = serde_json::from_str(include_str!(
            "../minisforumau/offer_details/v2/fixtures/product.json"
        ))
        .unwrap();
        for pointer in ["", "/priceRange/minVariantPrice"] {
            let mut changed = product.clone();
            changed
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unmodeledField".into(), Value::Bool(true));
            assert!(serde_json::from_value::<raw::Product>(changed).is_err());
        }
        let mut changed = product.clone();
        changed.as_object_mut().unwrap().remove("variants");
        assert!(serde_json::from_value::<raw::Product>(changed).is_err());
        let mut changed = product;
        changed["priceRange"]["minVariantPrice"]["amount"] = json!(12.34);
        assert!(serde_json::from_value::<raw::Product>(changed).is_err());
        let adjustment = json!({"__typename":"SellingPlanPercentagePriceAdjustment","adjustmentPercentage":12.5});
        let raw: raw::PriceAdjustmentValue = serde_json::from_value(adjustment).unwrap();
        let typed = PriceAdjustmentValue::try_from(raw).unwrap();
        let output = serde_json::to_value(typed).unwrap();
        assert_eq!(
            output,
            json!({"__typename":"SellingPlanPercentagePriceAdjustment","adjustment_percentage":12.5})
        );
        let decoded: PriceAdjustmentValue = serde_json::from_value(output.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), output);
    }

    #[test]
    fn extraction_requires_acquisition_time_and_context() {
        let product: Value = serde_json::from_str(include_str!(
            "../minisforumau/offer_details/v2/fixtures/product.json"
        ))
        .unwrap();
        let source = json!({"schema_version":2,"acquired_at":"2026-11-12T00:00:00Z",
            "context":{"country":"CA","language":"FR","currency":"CAD"},
            "product":product,"pages":[]});
        let raw: raw::RawOfferDetails = serde_json::from_value(source).unwrap();
        let extracted = ExtractedOfferDetails::try_from(raw.clone()).unwrap();
        let output = serde_json::to_value(extracted).unwrap();
        for field in ["acquired_at", "context"] {
            for null in [false, true] {
                let mut changed = output.clone();
                if null {
                    changed[field] = Value::Null;
                } else {
                    changed.as_object_mut().unwrap().remove(field);
                }
                assert!(serde_json::from_value::<ExtractedOfferDetails>(changed).is_err());
            }
            let mut changed = raw.clone();
            if field == "acquired_at" {
                changed.acquired_at = None;
            } else {
                changed.context = None;
            }
            let error = ExtractedOfferDetails::try_from(changed).unwrap_err();
            assert!(error.contains(field), "{error}");
        }
        let mut invalid = raw;
        invalid.acquired_at = Some("yesterday".into());
        assert!(
            ExtractedOfferDetails::try_from(invalid)
                .unwrap_err()
                .contains("acquired_at")
        );
    }
}
