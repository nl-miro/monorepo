//! Typed extraction of persisted initial-fill artifacts.
use shared::retailer::RetailerCode;

/// Parse the retailer-owned raw model and serialize its grouped extracted model.
/// Market context is retained; recorded pages remain in the raw artifact.
pub fn extract_v2_for(code: RetailerCode, json: &str) -> Result<String, String> {
    if retailers_anker::initial_fill::integration(code).is_some() {
        let artifact: retailers_anker::graphql::raw::RawOfferDetails =
            serde_json::from_str(json).map_err(|error| error.to_string())?;
        let extracted = retailers_anker::extracted::ExtractedOfferDetails::try_from(artifact)?;
        return serde_json::to_string(&extracted).map_err(|error| error.to_string());
    }
    if retailers_minisforum::initial_fill::integration(code).is_some() {
        let raw = serde_json::from_str::<retailers_minisforum::graphql::raw::RawOfferDetails>(json)
            .map_err(|e| e.to_string())?;
        let extracted = retailers_minisforum::extracted::ExtractedOfferDetails::try_from(raw)?;
        return serde_json::to_string(&extracted).map_err(|e| e.to_string());
    }
    if retailers_ugreen::initial_fill::integration(code).is_some() {
        let raw = serde_json::from_str::<retailers_ugreen::graphql::raw::RawOfferDetails>(json)
            .map_err(|e| e.to_string())?;
        let extracted = retailers_ugreen::extracted::ExtractedOfferDetails::try_from(raw)?;
        return serde_json::to_string(&extracted).map_err(|e| e.to_string());
    }
    Err(format!("no v2 extractor for {code:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    // Product fixtures come from each published retailer crate v0.1.0,
    // src/<retailer>/offer_details/v2/fixtures/product.json.
    #[test]
    fn parses_each_family_and_preserves_market_context() {
        for (code, product) in [
            (
                RetailerCode::MinisForumAu,
                include_str!("../tests/fixtures/v2/minisforumau.json"),
            ),
            (
                RetailerCode::UgreenCa,
                include_str!("../tests/fixtures/v2/ugreenca.json"),
            ),
        ] {
            let product: Value = serde_json::from_str(product).unwrap();
            let source = json!({"schema_version":2,"context":{"country":"CA","language":"FR","currency":"CAD"}, "acquired_at":"2026-11-12T00:00:00Z", "product": product, "pages": []});
            let output: Value =
                serde_json::from_str(&extract_v2_for(code, &source.to_string()).unwrap()).unwrap();
            assert_extracted_contract(&output, &source);
            let mut source = source;
            source["schema_version"] = json!(2);
            source["context"] = json!({"country":"AU","language":"EN","currency":"AUD"});
            let output: Value =
                serde_json::from_str(&extract_v2_for(code, &source.to_string()).unwrap()).unwrap();
            assert_extracted_contract(&output, &source);
            source["product"]["unexpected_field"] = json!(true);
            assert!(extract_v2_for(code, &source.to_string()).is_err());
        }
    }

    #[test]
    fn all_brands_require_acquisition_time_and_context() {
        for (code, fixture) in [
            (
                RetailerCode::AnkerAu,
                include_str!("../tests/fixtures/v2/ankerau.json"),
            ),
            (
                RetailerCode::MinisForumAu,
                include_str!("../tests/fixtures/v2/minisforumau.json"),
            ),
            (
                RetailerCode::UgreenCa,
                include_str!("../tests/fixtures/v2/ugreenca.json"),
            ),
        ] {
            let product: Value = serde_json::from_str(fixture).unwrap();
            let source = json!({"schema_version":2,"acquired_at":"2026-11-12T09:10:11Z",
                "context":{"country":"CA","language":"EN","currency":"CAD"},"product":product,"pages":[]});
            assert!(extract_v2_for(code, &source.to_string()).is_ok());
            for field in ["acquired_at", "context"] {
                for null in [false, true] {
                    let mut changed = source.clone();
                    if null {
                        changed[field] = Value::Null;
                    } else {
                        changed.as_object_mut().unwrap().remove(field);
                    }
                    if field == "context" {
                        changed["schema_version"] = json!(1);
                    }
                    assert!(
                        extract_v2_for(code, &changed.to_string())
                            .unwrap_err()
                            .contains(field)
                    );
                }
            }
            for value in [json!(123), json!("yesterday")] {
                let mut changed = source.clone();
                changed["acquired_at"] = value;
                assert!(extract_v2_for(code, &changed.to_string()).is_err());
            }
        }
    }

    #[test]
    fn every_initial_fill_retailer_has_a_v2_parser() {
        for code in RetailerCode::ALL {
            if crate::initial_fill_integration_for(code).is_some() {
                let error = extract_v2_for(code, "{}").unwrap_err();
                assert!(!error.contains("no v2 extractor"), "{code:?}: {error}");
            }
        }
    }

    #[test]
    fn rejects_unsupported_retailers_and_invalid_artifacts() {
        assert!(
            extract_v2_for(RetailerCode::MiCom, "{}")
                .unwrap_err()
                .contains("no v2 extractor")
        );
        assert!(extract_v2_for(RetailerCode::AnkerAu, "{}").is_err());
        assert!(extract_v2_for(RetailerCode::AnkerAu, "not json").is_err());
    }
}

#[cfg(test)]
mod anker_tests {
    use super::*;
    use serde_json::{Value, json};
    #[test]
    fn all_anker_storefronts_use_the_shared_extracted_model() {
        let product: Value = serde_json::from_str(include_str!(
            "../../retailers_anker/src/ankerau/offer_details/v2/fixtures/product.json"
        ))
        .unwrap();
        let source = json!({"schema_version":2,"context":{"country":"CA","language":"FR","currency":"CAD"},"acquired_at":"2026-11-12T00:00:00Z","product":product,"pages":[]});
        let mut count = 0;
        for code in RetailerCode::ALL {
            if retailers_anker::initial_fill::integration(code).is_none() {
                continue;
            }
            count += 1;
            let output: Value =
                serde_json::from_str(&extract_v2_for(code, &source.to_string()).unwrap()).unwrap();
            assert_extracted_contract(&output, &source);
        }
        assert_eq!(count, 17);
    }
}

#[cfg(test)]
pub(crate) fn assert_extracted_contract(output: &serde_json::Value, source: &serde_json::Value) {
    assert_eq!(
        output["product"]["identifiers"]["id"],
        source["product"]["id"]
    );
    assert_eq!(
        output["product"]["content"]["title"],
        source["product"]["title"]
    );
    assert_eq!(output["context"], source["context"]);
    assert_eq!(output["acquired_at"], source["acquired_at"]);
    assert!(output.get("pages").is_none());
    assert!(output.get("schema_version").is_none());
    assert!(output["product"].get("price_range").is_none());
    assert_eq!(
        output["product"]["variants"].as_array().unwrap().len(),
        source["product"]["variants"]["edges"]
            .as_array()
            .unwrap()
            .len()
    );
    let images = output["images"].as_array().unwrap();
    let ids: std::collections::HashSet<_> = images
        .iter()
        .map(|image| image["id"].as_str().unwrap())
        .collect();
    assert_eq!(images.len(), ids.len());
    for id in output["product"]["content"]["images"].as_array().unwrap() {
        assert!(ids.contains(id.as_str().unwrap()));
    }
}
