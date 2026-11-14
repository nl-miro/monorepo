//! Extracted offer-details model shared by Anker storefronts.
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
            "../ankercom/offer_details/v2/fixtures/product.json"
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
    }

    #[test]
    fn extraction_requires_acquisition_time_and_context() {
        let product: Value = serde_json::from_str(include_str!(
            "../ankercom/offer_details/v2/fixtures/product.json"
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

#[cfg(test)]
mod expanded_capture_tests {
    use super::prelude::*;
    use serde_json::Value;
    #[test]
    fn inventory_and_expanded_references_are_preserved() {
        let source: Value = serde_json::from_str(include_str!(
            "../ankercom/offer_details/v2/fixtures/expanded-product.json"
        ))
        .unwrap();
        let raw: crate::graphql::raw::Product = serde_json::from_value(source.clone()).unwrap();
        let typed = Product::try_from(raw).unwrap();
        let typed = serde_json::to_value(typed).unwrap();
        assert_eq!(typed["total_inventory"], source["totalInventory"]);
        assert_eq!(
            typed["metafields"].as_array().unwrap().len(),
            source["metafields"].as_array().unwrap().len()
        );
        let source_refs = source["_expandedReferences"].as_object().unwrap();
        let output_refs = typed["_expandedReferences"].as_object().unwrap();
        assert_eq!(output_refs.len(), source_refs.len());
        for (id, source_ref) in source_refs {
            assert_eq!(output_refs[id]["id"], source_ref["id"]);
            assert_eq!(
                output_refs[id]["metafields"].as_array().unwrap().len(),
                source_ref["metafields"].as_array().unwrap().len()
            );
        }
        let pairs = std::iter::once((&source["metafields"], &typed["metafields"])).chain(
            source_refs.iter().map(|(id, source_ref)| {
                (&source_ref["metafields"], &output_refs[id]["metafields"])
            }),
        );
        for (source_fields, output_fields) in pairs {
            for (source_field, output_field) in source_fields
                .as_array()
                .unwrap()
                .iter()
                .zip(output_fields.as_array().unwrap())
            {
                if source_field.is_null() {
                    assert!(output_field.is_null());
                    continue;
                }
                for field in [
                    "id",
                    "key",
                    "namespace",
                    "value",
                    "description",
                    "list",
                    "reference",
                    "references",
                ] {
                    assert_eq!(output_field[field], source_field[field], "{field}");
                }
                assert_eq!(output_field["type"], source_field["type"]);
                assert!(output_field.get("__typename").is_none());
            }
        }
        let decoded: Product = serde_json::from_value(typed.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), typed);
        assert!(
            typed["_expandedReferences"]
                .as_object()
                .is_some_and(|refs| !refs.is_empty())
        );
    }
}
