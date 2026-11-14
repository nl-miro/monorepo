//! Storefront configuration and compatibility adapters for ankerau.

use retailer_sauce::shopify::initial_fill::{GraphqlRequest, StorefrontConfig};
use serde_json::Value;
const CODE: &str = "ankerau";
const ALL_MARKETS: bool = true;
pub const CONFIG: StorefrontConfig = StorefrontConfig {
    acquire_token: true,
    token_required: true,
    storefront_url: "https://www.anker.com/au",
    graphql_url: "https://oneanker-au.myshopify.com/api/2026-10/graphql.json",
    token_market: Some("au"),
    token_environment: None,
    country: "AU",
    language: "EN",
    currency: "AUD",
};
pub use crate::graphql::Operation;

#[allow(unused_imports)]
pub use crate::graphql::{next_cursor, product_gid};
#[allow(non_upper_case_globals)]
pub const Integration: crate::graphql::Engine = crate::graphql::Engine {
    code: CODE,
    config: CONFIG,
    all_markets: ALL_MARKETS,
};
#[allow(dead_code)]
pub fn request(operation: &Operation) -> Result<GraphqlRequest, String> {
    Integration.request(operation)
}
#[allow(dead_code)]
pub fn validate_response(operation: &Operation, response: &Value) -> Result<(), String> {
    Integration.validate_response(operation, response)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn au_configuration_and_full_wip_response_match_typed_operations() {
        assert_eq!(CONFIG.storefront_url, "https://www.anker.com/au");
        assert_eq!(
            CONFIG.graphql_url,
            "https://oneanker-au.myshopify.com/api/2026-10/graphql.json"
        );
        let mut body: Value =
            serde_json::from_str(include_str!("fixtures/offer_details.json")).unwrap();
        // The WIP captured only __typename for presentations. Select their ID
        // too: graphql_client generates an empty enum for typename-only objects.
        // Keep the recorded fixture intact and supply only this newly selected
        // field for the schema compatibility check.
        fn presentation_ids(value: &mut Value) {
            match value {
                Value::Object(fields) => {
                    if fields.get("__typename").and_then(Value::as_str) == Some("MediaPresentation")
                    {
                        fields.insert("id".into(), json!("gid://shopify/MediaPresentation/1"));
                    }
                    for child in fields.values_mut() {
                        presentation_ids(child);
                    }
                }
                Value::Array(items) => items.iter_mut().for_each(presentation_ids),
                _ => {}
            }
        }
        presentation_ids(&mut body);
        let op = Operation::Details {
            id: body["data"]["product"]["id"].as_str().unwrap().into(),
        };
        validate_response(&op, &body).unwrap();
        assert_eq!(
            body["data"]["product"]["collections"]["pageInfo"]["hasNextPage"],
            true
        );
        let request = request(&op).unwrap();
        assert!(
            request
                .query
                .contains("country: $country, language: $language")
        );
        // Selling plans need a scope acquired storefront tokens usually lack.
        assert!(!request.query.contains("sellingPlan"));
        assert!(request.query.contains("storeAvailability"));
        assert_eq!(request.variables["id"], body["data"]["product"]["id"]);
    }
    #[test]
    fn catalog_selects_only_identifiers_and_preserves_cursor_variables() {
        let op = Operation::Catalog {
            first: 250,
            after: Some("cursor".into()),
        };
        let request = request(&op).unwrap();
        assert_eq!(
            request.variables,
            json!({"language":"EN","first":250,"after":"cursor"})
        );
        assert!(!request.query.contains("variants"));
        let body = json!({"data":{"products":{"edges":[{"cursor":"c","node":{"id":"gid://shopify/Product/1"}}],"pageInfo":{"hasNextPage":false,"endCursor":"c"}}}});
        validate_response(&op, &body).unwrap();
        assert!(
            validate_response(
                &op,
                &json!({"data":body["data"],"errors":[{"message":"denied"}]})
            )
            .is_err()
        );
        assert!(validate_response(&op, &json!({"data":null})).is_err());
        assert!(validate_response(&op, &json!({"data":{"products":{}}})).is_err());
    }

    #[test]
    fn variant_continuations_decode_fractional_coordinates() {
        let operation = Operation::VariantConnection {
            id: "gid://shopify/ProductVariant/1".into(),
            field: "storeAvailability".into(),
            after: Some("previous".into()),
        };
        let body = json!({"data":{"node":{
            "__typename":"ProductVariant", "id":"gid://shopify/ProductVariant/1",
            "storeAvailability":{
                "pageInfo":{"hasNextPage":false,"endCursor":"last"},
                "edges":[{"cursor":"last","node":{
                    "available":true,"pickUpTime":"Tomorrow","quantityAvailable":3,
                    "location":{"id":"gid://shopify/Location/1","name":"Sydney",
                        "address":{"formatted":["Sydney"],"latitude":-33.8688,"longitude":151.2093}}
                }}]
            }
        }}});
        validate_response(&operation, &body).unwrap();
    }
}

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
#[allow(unused_imports)]
use retailer_sauce::{
    initial_fill::{Http, InitialFillIntegration},
    shopify::{initial_fill::StorefrontToken, markets::MarketContext},
};
