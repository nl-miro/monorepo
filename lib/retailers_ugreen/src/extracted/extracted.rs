//! Extracted offer-details model owned by ugreenca, reshaped from `graphql::raw`.
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
    pub acquired_at: Option<DateTime<Utc>>,
    pub context: Option<retailer_sauce::shopify::markets::MarketContext>,
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
            "../ugreenca/offer_details/v2/fixtures/product.json"
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
        assert_eq!(
            serde_json::to_value(typed).unwrap(),
            json!({
                "__typename":"SellingPlanPercentagePriceAdjustment", "adjustment_percentage":12.5
            })
        );
    }
    #[test]
    fn historical_and_contextual_artifacts_remain_readable() {
        let product: Value = serde_json::from_str(include_str!(
            "../ugreenca/offer_details/v2/fixtures/product.json"
        ))
        .unwrap();
        let mut artifact = json!({"schema_version":1,"product":product,"pages":[]});
        let raw = serde_json::from_value::<raw::RawOfferDetails>(artifact.clone()).unwrap();
        let extracted = ExtractedOfferDetails::try_from(raw).unwrap();
        assert!(extracted.context.is_none());
        assert!(extracted.acquired_at.is_none());
        artifact["schema_version"] = json!(2);
        assert!(serde_json::from_value::<raw::RawOfferDetails>(artifact.clone()).is_err());
        artifact["context"] = json!({"country":"CA","language":"FR","currency":"CAD"});
        let raw = serde_json::from_value::<raw::RawOfferDetails>(artifact.clone()).unwrap();
        let extracted = ExtractedOfferDetails::try_from(raw).unwrap();
        assert_eq!(
            serde_json::to_value(extracted.context).unwrap(),
            artifact["context"]
        );
        artifact["schema_version"] = json!(1);
        assert!(serde_json::from_value::<raw::RawOfferDetails>(artifact).is_err());
        //         let error: ProductResponse = serde_json::from_value(
        //             json!({"data":{"product":null},"errors":[{"message":"missing"}]}),
        //         )
        //         .unwrap();
        //         assert!(error.data.unwrap().product.is_none());
    }
}
