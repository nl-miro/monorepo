//! Fact observations shared by persistence and outbox integration tests.
use serde_json::{Value, json};

use super::entity::OfferExport;

pub(crate) fn document() -> Value {
    let base = json!({
        "title":" Source charger ž ",
        "url":"https://shop.example/facts",
        "price":{"amount":1999,"currency":"AUD"},
        "images":["https://shop.example/facts.png"],
        "localization":{"target_market":{"kind":"country","code":"AU"},"language":"en"},
        "identifiers":{"sku":"SOURCE-SKU"}
    });
    let mut populated = base.clone();
    populated["price"] = json!({"amount":9_007_199_254_740_993_i64,"currency":"JPY"});
    populated["was_price"] = json!({"amount":9_007_199_254_740_994_i64,"currency":"JPY"});
    populated["availability"] = json!("available");
    populated["item_condition"] = json!("new");
    populated["content"] =
        json!({"description":" <p>Exact &amp; source ž \"quoted\" \\ text</p>\n"});
    let mut cleared = base.clone();
    for key in ["was_price", "availability", "item_condition"] {
        cleared[key] = Value::Null;
    }
    cleared["content"] = json!({"description":null});
    let mut raw = base.clone();
    raw["price"] = json!({"amount":0,"currency":"KRW"});
    raw["was_price"] = json!({"amount":1,"currency":"KRW"});
    raw["availability"] = json!({"other":"http://schema.org/OutOfStock"});
    raw["item_condition"] = json!({"other":"http://schema.org/NewCondition"});
    raw["content"] = json!({"description":" \n\t "});
    let mut empty = base.clone();
    empty["content"] = json!({"description":""});
    let mut variations = vec![populated, cleared, base, raw, empty];
    for (index, variation) in variations.iter_mut().enumerate() {
        variation["url"] = json!(format!("https://shop.example/facts?variant={index}"));
    }
    json!({"variations":variations})
}

pub(crate) fn assert_payloads(export: &OfferExport, document: &Value) {
    assert!(export.valid_schema());
    let variations = document["variations"].as_array().unwrap();
    assert_eq!(export.entries.len(), variations.len());
    assert!(export.exclusions.is_empty());
    for (index, entry) in export.entries.iter().enumerate() {
        assert_eq!(entry.variation_index, index);
        let payload = serde_json::to_value(&entry.payload).unwrap();
        let source = &variations[index];
        for field in ["price", "url"] {
            assert_eq!(payload[field], source[field]);
        }
        for field in ["was_price", "availability", "item_condition"] {
            assert_eq!(payload[field], source[field], "variant {index} {field}");
            assert!(payload.get(field).is_some());
        }
        assert_eq!(payload["content"]["title"], source["title"]);
        assert_eq!(
            payload["content"]["description"],
            source["content"]["description"]
        );
        assert!(payload.get("title").is_none());
    }
}

#[test]
fn persistence_fixture_preserves_missing_null_raw_and_exact_fact_values() {
    let document = document();
    let export =
        super::conversion::convert(super::conversion::tests::source(), &document.to_string())
            .unwrap();
    assert_payloads(&export, &document);
    let bytes = serde_json::to_string(&export).unwrap();
    let restored: OfferExport = serde_json::from_str(&bytes).unwrap();
    assert_payloads(&restored, &document);
    assert_eq!(serde_json::to_string(&restored).unwrap(), bytes);
}
