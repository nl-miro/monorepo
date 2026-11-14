//! Storefront configuration and compatibility adapters for ankervn.

use retailer_sauce::shopify::initial_fill::{GraphqlRequest, StorefrontConfig};
use serde_json::Value;
const CODE: &str = "ankervn";
const ALL_MARKETS: bool = true;
pub const CONFIG: StorefrontConfig = StorefrontConfig {
    acquire_token: true,
    token_required: true,
    storefront_url: "https://www.anker.com/vn",
    graphql_url: "https://ankervnm.myshopify.com/api/2026-10/graphql.json",
    token_market: Some("vn"),
    token_environment: None,
    country: "VN",
    language: "VI",
    currency: "VND",
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
mod integration_tests;

#[cfg(test)]
#[allow(unused_imports)]
use retailer_sauce::{
    initial_fill::{Http, InitialFillIntegration},
    shopify::{initial_fill::StorefrontToken, markets::MarketContext},
};
