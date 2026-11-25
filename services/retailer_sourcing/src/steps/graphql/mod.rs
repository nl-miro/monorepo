//! Shopify GraphQL workflow steps.

pub(super) mod acquire_storefront_token;
pub(super) mod extract_offer_details;
pub(super) mod offer_details;
pub(super) mod prepare_offer_imports;
pub(super) mod product_catalog_ids;
pub(super) mod request_initial_fill;
pub(super) mod structuralize_offer_details;

#[cfg(test)]
mod live_shopify_tests;
#[cfg(test)]
pub(super) mod tests;
