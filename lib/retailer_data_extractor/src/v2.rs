//! Typed extraction of persisted initial-fill artifacts.
use shared::retailer::RetailerCode;

/// Parse with the retailer-owned v2 source model and serialize the complete
/// artifact, retaining merged product data, market context and page evidence.
/// This stage does not normalize or structuralize the product.
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
    fn parses_each_family_and_preserves_market_and_page_evidence() {
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
            let source = json!({"schema_version": 1, "acquired_at":"2026-11-12T00:00:00Z", "product": product, "pages": []});
            let output: Value =
                serde_json::from_str(&extract_v2_for(code, &source.to_string()).unwrap()).unwrap();
            // Extraction is a one-to-one copy of the raw artifact.
            assert_eq!(output, source);
            let mut source = source;
            source["schema_version"] = json!(2);
            source["context"] = json!({"country":"AU","language":"EN","currency":"AUD"});
            let output: Value =
                serde_json::from_str(&extract_v2_for(code, &source.to_string()).unwrap()).unwrap();
            assert_eq!(output, source);
            source["product"]["unexpected_field"] = json!(true);
            assert!(extract_v2_for(code, &source.to_string()).is_err());
        }
    }

    #[test]
    fn minisforum_and_ugreen_preserve_optional_acquisition_time() {
        for code in RetailerCode::ALL {
            let product = if retailers_minisforum::initial_fill::integration(code).is_some() {
                include_str!("../tests/fixtures/v2/minisforumau.json")
            } else if retailers_ugreen::initial_fill::integration(code).is_some() {
                include_str!("../tests/fixtures/v2/ugreenca.json")
            } else {
                continue;
            };
            let product: Value = serde_json::from_str(product).unwrap();
            for version in [1, 2] {
                let mut source = json!({"schema_version":version,"product":product,"pages":[]});
                if version == 2 {
                    source["context"] = json!({"country":"CA","language":"EN","currency":"CAD"});
                }
                for timestamp in [
                    None,
                    Some(Value::Null),
                    Some(json!("2026-11-12T09:10:11.123Z")),
                ] {
                    source.as_object_mut().unwrap().remove("acquired_at");
                    if let Some(timestamp) = timestamp {
                        source["acquired_at"] = timestamp;
                    }
                    let output: Value =
                        serde_json::from_str(&extract_v2_for(code, &source.to_string()).unwrap())
                            .unwrap();
                    assert_eq!(output["acquired_at"], source["acquired_at"], "{code:?}");
                    if source["acquired_at"].is_null() {
                        assert_eq!(output.get("acquired_at"), Some(&Value::Null));
                    }
                }
                source["acquired_at"] = json!(123);
                assert!(extract_v2_for(code, &source.to_string()).is_err());
                source["acquired_at"] = json!("2026-11-12T09:10:11Z");
                source["unknown_metadata"] = json!(true);
                assert!(extract_v2_for(code, &source.to_string()).is_err());
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
        let source = json!({"schema_version":1,"acquired_at":"2026-11-12T00:00:00Z","product":product,"pages":[]});
        let mut count = 0;
        for code in RetailerCode::ALL {
            if retailers_anker::initial_fill::integration(code).is_none() {
                continue;
            }
            count += 1;
            let output: Value =
                serde_json::from_str(&extract_v2_for(code, &source.to_string()).unwrap()).unwrap();
            // Extraction is a one-to-one copy of the raw artifact.
            assert_eq!(output, source, "{code:?}");
        }
        assert_eq!(count, 17);
    }
}
