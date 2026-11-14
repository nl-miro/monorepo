//! Storefront configuration and compatibility adapters for ankerca.

use retailer_sauce::shopify::initial_fill::{GraphqlRequest, StorefrontConfig};
use serde_json::Value;
const CODE: &str = "ankerca";
const ALL_MARKETS: bool = true;
pub const CONFIG: StorefrontConfig = StorefrontConfig {
    acquire_token: true,
    token_required: true,
    storefront_url: "https://www.anker.com/ca",
    graphql_url: "https://anker-canada.myshopify.com/api/2026-10/graphql.json",
    token_market: Some("ca"),
    token_environment: None,
    country: "CA",
    language: "EN",
    currency: "CAD",
};
pub use crate::graphql::Operation;

#[allow(unused_imports)]
pub use crate::graphql::{markets, next_cursor, product_gid};
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
    fn every_canadian_operation_including_continuations_requests_english() {
        assert_eq!(CONFIG.storefront_url, "https://www.anker.com/ca");
        assert_eq!(CONFIG.token_market, Some("ca"));
        assert_eq!(CONFIG.currency, "CAD");
        assert_eq!(CONFIG.language, "EN");
        let mut operations = vec![
            Operation::Catalog {
                first: 250,
                after: Some("catalog-next".into()),
            },
            Operation::Details {
                id: "gid://shopify/Product/1".into(),
            },
        ];
        for field in ["collections", "images", "media", "variants"] {
            operations.push(Operation::ProductConnection {
                id: "gid://shopify/Product/1".into(),
                field: field.into(),
                after: Some("next".into()),
            });
        }
        for field in [
            "components",
            "groupedBy",
            "quantityPriceBreaks",
            "storeAvailability",
        ] {
            operations.push(Operation::VariantConnection {
                id: "gid://shopify/ProductVariant/1".into(),
                field: field.into(),
                after: Some("next".into()),
            });
        }
        for operation in operations {
            let request = request(&operation).unwrap();
            assert!(
                request
                    .query
                    .contains("@inContext(country: $country, language: $language)")
            );
            assert!(request.variables.get("country").is_none());
            assert_eq!(request.variables["language"], "EN");
            assert!(!request.query.contains("language: FR"));
        }
    }

    #[test]
    fn canadian_catalog_uses_typed_response_validation() {
        let operation = Operation::Catalog {
            first: 250,
            after: None,
        };
        let response = json!({"data":{"products":{"edges":[{"cursor":"p1","node":{"id":"gid://shopify/Product/1"}}],"pageInfo":{"hasNextPage":false,"endCursor":"p1"}}}});
        validate_response(&operation, &response).unwrap();
        assert!(
            validate_response(
                &operation,
                &json!({"data":response["data"],"errors":[{"message":"denied"}]})
            )
            .is_err()
        );
        assert!(validate_response(&operation, &json!({"data":{"products":{}}})).is_err());
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
