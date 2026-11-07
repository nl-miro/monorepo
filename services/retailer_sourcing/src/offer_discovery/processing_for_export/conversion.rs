use std::collections::HashSet;

use money::Money;
use retailer_sauce::availability::Availability;
use retailer_sauce::identifier::{
    Gtin, OfferIdentifiers, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku,
};
use retailer_sauce::item_condition::ItemCondition;
use retailer_sauce::json::nullable_object;
use retailer_sauce::localization::Localization;
use retailer_sauce::structuralized::content::{Content, observation};
use serde::Deserialize;
use uuid::Uuid;

use super::entity::{
    ContentNextGeneration, ExclusionReason, ExportEntry, ExportExclusion, ExportSource,
    OfferExport, OfferImport,
};

#[derive(Deserialize)]
struct StructuralizedDocument {
    variations: Vec<Variation>,
}

#[derive(Deserialize)]
struct Variation {
    #[serde(default, deserialize_with = "nullable_object")]
    was_price: Option<Money>,
    #[serde(default)]
    availability: Option<Availability>,
    #[serde(default)]
    item_condition: Option<ItemCondition>,
    #[serde(default, deserialize_with = "observation")]
    content: Option<Content>,

    #[serde(default)]
    identifiers: VariationIdentifiers,
    #[serde(default)]
    localization: Localization,
    title: String,
    url: String,
    price: Option<Money>,
    #[serde(default = "empty_images")]
    images: serde_json::Value,
}

#[derive(Default, Deserialize)]
struct VariationIdentifiers {
    gtin: Option<Gtin>,
    sku: Option<Sku>,
    shopify_product_gid: Option<ShopifyProductGid>,
    shopify_product_variation_gid: Option<ShopifyProductVariationGid>,
    shopify_handle: Option<ShopifyHandle>,
}
impl VariationIdentifiers {
    fn into_identifiers(self) -> OfferIdentifiers {
        OfferIdentifiers {
            gtin: self.gtin,
            sku: self.sku,
            shopify_product_gid: self.shopify_product_gid,
            shopify_product_variation_gid: self.shopify_product_variation_gid,
            shopify_handle: self.shopify_handle,
        }
    }
}

fn err_missing_scraped_at() -> ConversionError {
    ConversionError("scraped_at is required for export".into())
}

fn err_missing_localization(index: usize, field: &str) -> ConversionError {
    ConversionError(format!("variation {index} requires {field}"))
}

fn empty_images() -> serde_json::Value {
    serde_json::json!([])
}

#[derive(Debug, thiserror::Error)]
#[error("offer export conversion failed: {0}")]
pub struct ConversionError(String);

pub(super) fn convert(source: ExportSource, body: &str) -> Result<OfferExport, ConversionError> {
    let scraped_at = source
        .storage
        .scraped_at
        .ok_or_else(err_missing_scraped_at)?;
    let input: StructuralizedDocument =
        serde_json::from_str(body).map_err(|error| ConversionError(error.to_string()))?;
    let mut entries = Vec::new();
    let mut exclusions = Vec::new();
    let mut urls = HashSet::new();
    for (variation_index, variation) in input.variations.into_iter().enumerate() {
        variation.guard(variation_index)?;
        let localization = variation
            .localization
            .for_write()
            .map_err(ConversionError)?;
        if !urls.insert(variation.url.clone()) {
            return Err(ConversionError(format!(
                "variation {variation_index} repeats a retailer/URL identity"
            )));
        }
        let images = {
            let (images, skipped) = shared::images::filter(
                serde_json::from_value(variation.images)
                    .map_err(|error| ConversionError(error.to_string()))?,
            );
            tracing::info!(skipped_images = skipped, "filtered offer images");
            images
        };
        let identifiers = variation.identifiers.into_identifiers();
        identifiers.guard().map_err(ConversionError)?;
        let Some(price) = variation.price else {
            exclusions.push(ExportExclusion {
                variation_index,
                url: variation.url,
                reason: ExclusionReason::MissingPrice,
            });
            continue;
        };
        let content = ContentNextGeneration {
            title: variation.title,
            description: variation.content.and_then(|value| value.description),
        };
        entries.push(ExportEntry {
            id: Uuid::now_v7(),
            variation_index,
            payload: OfferImport {
                was_price: variation.was_price,
                availability: variation.availability,
                item_condition: variation.item_condition,
                content,
                identifiers: identifiers.into(),
                target_market: localization
                    .target_market
                    .ok_or_else(|| err_missing_localization(variation_index, "target_market"))?,
                language: localization
                    .language
                    .ok_or_else(|| err_missing_localization(variation_index, "language"))?,
                scraped_at,
                images,
                retailer_code: source.retailer_code,
                price,
                url: variation.url,
            },
        });
    }
    Ok(OfferExport {
        id: Uuid::now_v7(),
        source,
        entries,
        exclusions,
    })
}

impl Variation {
    fn guard(&self, index: usize) -> Result<(), ConversionError> {
        if self.title.trim().is_empty() || self.title.contains('\0') {
            return Err(ConversionError(format!(
                "variation {index} requires a nonblank title without NUL"
            )));
        }
        if self.url.len() > 2048
            || self.url.trim() != self.url
            || self.url.chars().any(char::is_control)
            || !url::Url::parse(&self.url)
                .is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.has_host())
        {
            return Err(ConversionError(format!(
                "variation {index} requires an absolute HTTP(S) URL of at most 2048 bytes without padding or controls"
            )));
        }
        if self.price.as_ref().is_some_and(|price| price.amount() < 0) {
            return Err(ConversionError(format!(
                "variation {index} requires a nonnegative price"
            )));
        }
        if let Some(was) = self.was_price.as_ref()
            && let Some(price) = &self.price
            && (was.currency() != price.currency() || was.amount() <= price.amount())
        {
            return Err(ConversionError(format!(
                "variation {index} has an invalid comparison price"
            )));
        }
        if let Some(value) = self.availability.as_ref() {
            value.guard().map_err(ConversionError)?;
        }
        if let Some(value) = self.item_condition.as_ref() {
            value.guard().map_err(ConversionError)?;
        }
        if let Some(value) = &self.content {
            value.guard().map_err(ConversionError)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::platform::object_storage::OfferStorageMetadata;
    use bodul_shared::retailer::RetailerCode;
    use serde_json::{Value, json};

    pub(crate) const ANKER: &str =
        include_str!("../../../tests/fixtures/offer_export/ankerau.json");
    pub(crate) const UGREEN: &str =
        include_str!("../../../tests/fixtures/offer_export/ugreenca.json");
    pub(crate) const MINISFORUM: &str =
        include_str!("../../../tests/fixtures/offer_export/minisforumus.json");

    pub(crate) fn source() -> ExportSource {
        ExportSource {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://shop.example/Charger".into(),
            storage: OfferStorageMetadata {
                scraped_at: Some("2026-10-26T12:00:00.123456Z".parse().unwrap()),
                object_storage_id: Uuid::now_v7(),
                external_id: Uuid::now_v7(),
                content_size: ANKER.len() as u64,
            },
        }
    }

    #[test]
    fn preserves_priced_variations_and_matches_the_offers_contract_fixture() {
        let output = convert(source(), ANKER).unwrap();
        let expected: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/offer_export/import_payload.json"
        ))
        .unwrap();
        assert_eq!(
            serde_json::to_value(&output.entries[0].payload).unwrap(),
            expected
        );
        assert_eq!(output.entries.len(), 2);
        assert_eq!(output.entries[1].payload.price.amount(), 0);
        assert_eq!(output.entries[1].variation_index, 1);
        assert_ne!(output.entries[0].id, output.entries[1].id);
        assert!(
            output
                .entries
                .iter()
                .all(|entry| entry.id.get_version_num() == 7)
        );
        assert!(output.exclusions.is_empty());
    }

    #[test]
    fn records_missing_price_and_keeps_unavailable_priced_variations() {
        let mut source = source();
        source.retailer_code = RetailerCode::UgreenCa;
        let output = convert(source, UGREEN).unwrap();
        assert_eq!(output.entries.len(), 1);
        assert_eq!(
            output.entries[0].payload.retailer_code,
            RetailerCode::UgreenCa
        );
        assert_eq!(
            output.exclusions,
            vec![ExportExclusion {
                variation_index: 1,
                url: "https://ca.example/products/hub?variant=2".into(),
                reason: ExclusionReason::MissingPrice,
            }]
        );
        let mut source = output.source;
        source.retailer_code = RetailerCode::MinisForumUs;
        let output = convert(source, MINISFORUM).unwrap();
        assert_eq!(output.entries[0].payload.price.amount(), 59900);
        assert_eq!(
            output.entries[0].payload.retailer_code,
            RetailerCode::MinisForumUs
        );
    }

    #[test]
    fn supports_empty_exports_and_missing_price_fields() {
        let output = convert(source(), r#"{"variations":[]}"#).unwrap();
        assert!(output.entries.is_empty());
        assert!(output.exclusions.is_empty());
        let mut input: Value = serde_json::from_str(MINISFORUM).unwrap();
        input["variations"][0]
            .as_object_mut()
            .unwrap()
            .remove("price");
        let output = convert(source(), &input.to_string()).unwrap();
        assert!(output.entries.is_empty());
        assert_eq!(output.exclusions.len(), 1);
    }

    #[test]
    fn preserves_exact_integer_money_including_zero_decimal_currencies() {
        for currency in ["AUD", "JPY", "KRW", "VND"] {
            for amount in [0, 1, i64::MAX] {
                let mut input: Value = serde_json::from_str(MINISFORUM).unwrap();
                input["variations"][0]["price"] = json!({"amount":amount,"currency":currency});
                let output = convert(source(), &input.to_string()).unwrap();
                assert_eq!(
                    serde_json::to_value(&output.entries[0].payload.price).unwrap(),
                    json!({"amount":amount,"currency":currency})
                );
            }
        }
    }

    #[test]
    fn rejects_invalid_input_before_any_preparation() {
        for input in ["{}", "null", "[]", r#"{"variations":null}"#, "not JSON"] {
            assert!(convert(source(), input).is_err(), "{input}");
        }
        for (field, value) in [
            ("title", json!("  ")),
            ("title", json!("nul\u{0}")),
            ("title", json!(7)),
            ("url", json!("/relative")),
            ("url", json!("ftp://shop.example/product")),
            ("url", json!(" https://shop.example/product")),
            ("url", json!("https://shop.example/\nproduct")),
            (
                "url",
                json!(format!("https://shop.example/{}", "a".repeat(2048))),
            ),
            ("price", json!({"amount":-1,"currency":"AUD"})),
            ("price", json!({"amount":1.5,"currency":"AUD"})),
            ("price", json!({"amount":"1","currency":"AUD"})),
            ("price", json!({"amount":1,"currency":"BAD"})),
        ] {
            let mut input: Value = serde_json::from_str(MINISFORUM).unwrap();
            input["variations"][0][field] = value;
            assert!(convert(source(), &input.to_string()).is_err(), "{input}");
        }
        let mut input: Value = serde_json::from_str(ANKER).unwrap();
        input["variations"][1]["url"] = input["variations"][0]["url"].clone();
        assert!(
            convert(source(), &input.to_string())
                .unwrap_err()
                .to_string()
                .contains("repeats")
        );
    }

    #[test]
    fn images_are_ordered_and_filtered() {
        let mut input: Value = serde_json::from_str(ANKER).unwrap();
        let a = "https://shop.example/A?q=1&x=2#front";
        let b = "http://shop.example/B";
        input["variations"][0]["images"] =
            json!([a, a, "/relative", b, "https://user@shop.example/a"]);
        let current = convert(source(), &input.to_string()).unwrap();
        assert_eq!(current.entries[0].payload.images, vec![a, b]);
        assert_eq!(current.entries[1].payload.images, Vec::<String>::new());
        assert!(current.valid_schema());
        for images in [json!([]), json!(["/invalid", "javascript:alert(1)"])] {
            input["variations"][0]["images"] = images;
            assert_eq!(
                convert(source(), &input.to_string()).unwrap().entries[0]
                    .payload
                    .images,
                Vec::<String>::new()
            );
        }
        for images in [
            json!(null),
            json!("https://shop.example/A"),
            json!([1]),
            json!({}),
        ] {
            input["variations"][0]["images"] = images;
            assert!(convert(source(), &input.to_string()).is_err());
        }
        input["variations"][0]
            .as_object_mut()
            .unwrap()
            .remove("images");
        assert_eq!(
            convert(source(), &input.to_string()).unwrap().entries[0]
                .payload
                .images,
            Vec::<String>::new()
        );
    }
}

#[cfg(test)]
mod scrape_tests {
    use super::*;
    #[test]
    fn preserves_scrape_time_and_rejects_missing_or_inconsistent_time() {
        let mut source = tests::source();
        source.storage.scraped_at =
            Some(chrono::DateTime::from_timestamp_micros(1_700_000_000_123_456).unwrap());
        let snapshot = convert(source.clone(), tests::ANKER).unwrap();
        assert!(snapshot.valid_schema());
        for entry in &snapshot.entries {
            assert_eq!(Some(entry.payload.scraped_at), source.storage.scraped_at);
        }
        source.storage.scraped_at = None;
        assert!(convert(source, tests::ANKER).is_err());
        let mut invalid = snapshot.clone();
        invalid.entries[0].payload.scraped_at += chrono::Duration::seconds(1);
        assert!(!invalid.valid_schema());
    }
}

#[cfg(test)]
mod extended_contract_tests {
    use super::*;
    #[test]
    fn exports_reject_missing_market_or_language() {
        for field in ["target_market", "language"] {
            for remove in [true, false] {
                let mut body: serde_json::Value = serde_json::from_str(tests::ANKER).unwrap();
                if remove {
                    body["variations"][0]["localization"]
                        .as_object_mut()
                        .unwrap()
                        .remove(field);
                } else {
                    body["variations"][0]["localization"][field] = serde_json::Value::Null;
                }
                let error = convert(tests::source(), &body.to_string()).unwrap_err();
                assert!(error.to_string().contains(&format!("requires {field}")));
            }
        }
    }

    #[test]
    fn localization_policy_is_not_reapplied_to_exports() {
        for localization in [
            serde_json::json!({"target_market":{"kind":"country","code":"IT"},"language":"it"}),
            serde_json::json!({"target_market":{"kind":"region","code":"eu"},"language":"en"}),
        ] {
            let mut body: serde_json::Value = serde_json::from_str(tests::ANKER).unwrap();
            for variation in body["variations"].as_array_mut().unwrap() {
                variation["localization"] = localization.clone();
            }
            let export = convert(tests::source(), &body.to_string()).unwrap();
            assert!(!export.entries.is_empty());
            for entry in &export.entries {
                assert_eq!(
                    serde_json::json!({"target_market":entry.payload.target_market,"language":entry.payload.language}),
                    localization
                );
            }
            let bytes = serde_json::to_vec(&export).unwrap();
            let replay: OfferExport = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(serde_json::to_vec(&replay).unwrap(), bytes);
        }
    }

    #[test]
    fn keeps_exact_identifiers() {
        let body = serde_json::json!({"variations":[{"title":" USB charger ž ","url":"https://shop.example/Charger?q=red&variant=1#details","price":{"amount":1999,"currency":"AUD"},"images":["https://example.com/a.png"],"localization":{"target_market":{"kind":"country","code":"AU"},"language":"en-AU"},"identifiers":{"gtin":"0012345678905","sku":"A1-BLACK","shopify_product_gid":"gid://shopify/Product/1","shopify_product_variation_gid":"gid://shopify/ProductVariant/2","shopify_handle":"usb-charger"}}]});
        let mut source = tests::source();
        source.storage.scraped_at = Some("2026-10-26T12:00:00.123456Z".parse().unwrap());
        let snapshot = convert(source.clone(), &body.to_string()).unwrap();
        assert!(snapshot.valid_schema());
        assert_eq!(
            serde_json::to_value(&snapshot.entries[0].payload).unwrap(),
            {
                let mut expected: serde_json::Value = serde_json::from_str(include_str!(
                    "../../../tests/fixtures/offer_export/import_with_identifiers.json"
                ))
                .unwrap();
                for field in ["was_price", "availability", "item_condition"] {
                    expected[field] = serde_json::Value::Null;
                }
                expected
            }
        );
        for value in [
            serde_json::json!(null),
            serde_json::json!({"gtin":""}),
            serde_json::json!({"sku":123}),
            serde_json::json!({"shopify_handle":"\0"}),
        ] {
            let mut invalid = body.clone();
            invalid["variations"][0]["identifiers"] = value;
            assert!(convert(source.clone(), &invalid.to_string()).is_err());
        }
        for fixture in [tests::ANKER, tests::UGREEN, tests::MINISFORUM] {
            let snapshot = convert(source.clone(), fixture).unwrap();
            assert!(snapshot.valid_schema());
            assert!(
                snapshot
                    .entries
                    .iter()
                    .all(|entry| entry.payload.identifiers.guard().is_ok())
            );
        }
    }
}

#[cfg(test)]
mod facts_tests {
    use super::*;
    use serde_json::{Value, json};

    fn document() -> Value {
        let payload: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/offer_export/import_with_facts.json"
        ))
        .unwrap();
        let mut variation = payload.clone();
        variation["title"] = payload["content"]["title"].clone();
        variation["content"] = json!({"description":payload["content"]["description"]});
        variation["localization"] =
            json!({"target_market":payload["target_market"],"language":payload["language"]});
        json!({"variations":[variation]})
    }

    #[test]
    fn export_preserves_facts_and_round_trips() {
        let mut source = tests::source();
        source.storage.scraped_at = Some("2026-10-26T12:00:00.123456Z".parse().unwrap());
        let fixture = include_str!("../../../tests/fixtures/offer_export/import_with_facts.json");
        let snapshot = convert(source.clone(), &document().to_string()).unwrap();
        assert!(snapshot.valid_schema());
        let payload = serde_json::to_value(&snapshot.entries[0].payload).unwrap();
        let expected: Value = serde_json::from_str(fixture).unwrap();
        assert_eq!(payload, expected);
        let bytes = serde_json::to_vec(&snapshot).unwrap();
        let restored: OfferExport = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    }

    #[test]
    fn missing_and_null_optional_facts_are_unknown() {
        let missing = convert(tests::source(), tests::ANKER).unwrap();
        assert!(missing.valid_schema());
        let payload = &missing.entries[0].payload;
        assert!(payload.was_price.is_none());
        assert!(payload.availability.is_none());
        assert!(payload.item_condition.is_none());
        assert_eq!(payload.content.description, None);
        let mut body = document();
        for key in ["was_price", "availability", "item_condition"] {
            body["variations"][0][key] = Value::Null;
        }
        body["variations"][0]["content"] = json!({"description":null});
        let snapshot = convert(tests::source(), &body.to_string()).unwrap();
        assert!(snapshot.valid_schema());
        let payload = &snapshot.entries[0].payload;
        assert_eq!(payload.was_price, None);
        assert_eq!(payload.availability, None);
        assert_eq!(payload.item_condition, None);
        assert_eq!(payload.content.description, None);
    }

    #[test]
    fn malformed_facts_fail_before_snapshot_preparation() {
        for (field, invalid) in [
            ("was_price", json!({"amount":1999,"currency":"AUD"})),
            ("was_price", json!({"amount":2999,"currency":"USD"})),
            ("availability", json!({"other":"  "})),
            ("item_condition", json!("guessed_new")),
            ("was_price", json!([2999, "AUD"])),
            ("content", json!([null])),
            ("content", json!(null)),
            ("content", json!({})),
            ("content", json!({"description":42})),
            ("content", json!({"description":"bad\u{0}"})),
        ] {
            let mut body = document();
            body["variations"][0][field] = invalid;
            assert!(
                convert(tests::source(), &body.to_string()).is_err(),
                "{body}"
            );
        }
    }
}

#[cfg(test)]
mod regional_tests {
    use super::*;
    #[test]
    fn exports_use_region_codes() {
        let mut body: serde_json::Value = serde_json::from_str(tests::ANKER).unwrap();
        for variation in body["variations"].as_array_mut().unwrap() {
            variation["localization"] =
                serde_json::json!({"target_market":{"kind":"region","code":"eu"},"language":"en"});
        }
        let snapshot = convert(tests::source(), &body.to_string()).unwrap();
        assert!(snapshot.valid_schema());
        for entry in snapshot.entries {
            let value = serde_json::to_value(entry.payload.target_market).unwrap();
            let expected = serde_json::json!({"kind":"region","code":"eu"});
            assert_eq!(value, expected);
        }
    }
}
