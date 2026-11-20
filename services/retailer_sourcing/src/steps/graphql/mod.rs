//! Shopify GraphQL initial-fill steps, in execution order.

pub(crate) mod acquire_storefront_token;
pub(crate) mod extract_offer_details;
pub(crate) mod initialize_retailer_fill;
pub(crate) mod offer_details;
pub(crate) mod prelude;
mod prepare_offer_imports;
pub(crate) mod product_catalog_ids;
pub(crate) mod structuralize_offer_details;

use retailer_data_extractor::get_storefront_config;
use retailer_sauce::shopify::graphql::Engine;
use shared::retailer::RetailerCode;

/// The Shopify engine for a configured retailer storefront.
pub(crate) fn storefront_engine(retailer: RetailerCode) -> Result<Engine, String> {
    get_storefront_config(retailer)
        .map(Engine)
        .ok_or_else(|| "initial fill is not configured".into())
}
