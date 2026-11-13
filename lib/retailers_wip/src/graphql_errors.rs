//! GraphQL diagnostics shared by the brand engines. Never include request credentials or data.
use serde_json::{Map, Value};

/// Inspect errors before typed data: partial data must not hide the upstream error.
pub fn check(operation: &str, body: &Value) -> Result<(), String> {
    let Some(errors) = body.get("errors").filter(|value| !value.is_null()) else {
        return Ok(());
    };
    let errors = errors.as_array().ok_or_else(|| {
        format!("Storefront {operation} returned a malformed GraphQL errors field")
    })?;
    if errors.is_empty() {
        return Ok(());
    }
    let diagnostics: Vec<Value> = errors
        .iter()
        .map(|error| {
            let mut diagnostic = Map::new();
            for key in ["message", "path", "locations"] {
                if let Some(value) = error.get(key) {
                    diagnostic.insert(key.into(), value.clone());
                }
            }
            let mut extensions = Map::new();
            for key in [
                "code",
                "requiredAccess",
                "documentation",
                "cost",
                "maxCost",
                "throttleStatus",
            ] {
                if let Some(value) = error.get("extensions").and_then(|value| value.get(key)) {
                    extensions.insert(key.into(), value.clone());
                }
            }
            if !extensions.is_empty() {
                diagnostic.insert("extensions".into(), Value::Object(extensions));
            }
            if diagnostic.is_empty() {
                diagnostic.insert(
                    "message".into(),
                    Value::String("Unrecognized GraphQL error entry".into()),
                );
            }
            Value::Object(diagnostic)
        })
        .collect();
    Err(format!(
        "Storefront {operation} returned GraphQL errors: {}",
        Value::Array(diagnostics)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn preserves_multiple_errors_and_excludes_unrelated_payload_fields() {
        let body = json!({"data":{"private":"do not log data"},"errors":[
            {"message":"Access denied", "path":["product","variants",0,"sellingPlanAllocations"],
             "locations":[{"line":3,"column":5}], "extensions":{"code":"ACCESS_DENIED","requiredAccess":"unauthenticated_read_selling_plans","headers":{"authorization":"do not log headers"}}},
            {"message":"Query too large","extensions":{"code":"MAX_COST_EXCEEDED","cost":1001,"maxCost":1000}}
        ]});
        let error = check("OfferDetails", &body).unwrap_err();
        for value in [
            "OfferDetails",
            "Access denied",
            "sellingPlanAllocations",
            "ACCESS_DENIED",
            "unauthenticated_read_selling_plans",
            "locations",
            "Query too large",
            "MAX_COST_EXCEEDED",
            "1001",
            "1000",
        ] {
            assert!(error.contains(value), "missing {value}: {error}");
        }
        assert!(!error.contains("do not log"));
        let diagnostics: Value =
            serde_json::from_str(error.split_once("errors: ").unwrap().1).unwrap();
        assert_eq!(diagnostics.as_array().unwrap().len(), 2);
        assert_eq!(diagnostics[0]["path"], body["errors"][0]["path"]);
    }
    #[test]
    fn empty_errors_are_not_confused_with_malformed_envelopes() {
        for body in [json!({}), json!({"errors":null}), json!({"errors":[]})] {
            assert!(check("CatalogIdentifiers", &body).is_ok());
        }
        assert!(
            check("CatalogIdentifiers", &json!({"errors":{}}))
                .unwrap_err()
                .contains("malformed")
        );
        assert!(
            check("CatalogIdentifiers", &json!({"errors":[null]}))
                .unwrap_err()
                .contains("Unrecognized")
        );
    }
}
