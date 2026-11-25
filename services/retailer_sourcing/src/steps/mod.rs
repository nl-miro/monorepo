//! Workflow steps.

pub mod prelude;

mod graphql;
mod html;
#[path = "processing-offers/mod.rs"]
mod processing_offers;
mod sitemap;

#[cfg(test)]
mod postgres_tests;

use self::graphql::acquire_storefront_token::Handler as AcquireStorefrontTokenHandler;
use self::graphql::extract_offer_details::Handler as ExtractOfferDetailsHandler;
use self::graphql::offer_details::Handler as OfferDetailsHandler;
use self::graphql::prepare_offer_imports::Listener as PrepareOfferImportsListener;
use self::graphql::product_catalog_ids::Handler as ProductCatalogIdsHandler;
use self::graphql::structuralize_offer_details::Handler as StructuralizeOfferDetailsHandler;
use crate::platform::{Platform, bus::Bus};

pub(crate) fn register_shopify_steps(bus: &mut Bus, platform: &Platform) {
    bus.handle(AcquireStorefrontTokenHandler {
        fetcher: platform.http.clone(),
    });
    bus.handle(ProductCatalogIdsHandler {
        fetcher: platform.offer_http.clone(),
    });
    bus.handle(OfferDetailsHandler {
        fetcher: platform.offer_http.clone(),
        store: platform.object_storage.clone(),
    });
    bus.handle(ExtractOfferDetailsHandler {
        storage: platform.object_storage.clone(),
    });
    bus.handle(StructuralizeOfferDetailsHandler {
        storage: platform.object_storage.clone(),
    });
    bus.listen(PrepareOfferImportsListener {
        pool: platform.db.clone(),
        storage: platform.object_storage.clone(),
    });
}
