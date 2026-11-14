//! Storefront configuration and compatibility adapters for ugreenkr.

#[cfg(test)]
use retailer_sauce::shopify::initial_fill::GraphqlRequest;
use retailer_sauce::shopify::initial_fill::StorefrontConfig;
#[cfg(test)]
use serde_json::Value;
const CODE: &str = "ugreenkr";
pub const CONFIG: StorefrontConfig = StorefrontConfig {
    acquire_token: true,
    token_required: false,
    storefront_url: "https://kr.ugreen.com",
    graphql_url: "https://ugreen-kr.myshopify.com/api/2026-10/graphql.json",
    token_market: None,
    token_environment: None,
    country: "KR",
    language: "KO",
    currency: "KRW",
};
#[cfg(test)]
use crate::graphql::Operation;

#[allow(unused_imports)]
pub use crate::graphql::{next_cursor, product_gid};
#[allow(non_upper_case_globals)]
pub const Integration: crate::graphql::Engine = crate::graphql::Engine {
    code: CODE,
    config: CONFIG,
};
#[cfg(test)]
mod integration_tests;

#[cfg(test)]
#[allow(unused_imports)]
use retailer_sauce::{
    initial_fill::{Http, InitialFillIntegration},
    shopify::{initial_fill::StorefrontToken, markets::MarketContext},
};
