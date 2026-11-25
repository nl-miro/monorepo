//! Registration of the Shopify GraphQL initial-fill steps.

use super::acquire_storefront_token::prelude::register as register_token;
use super::extract_offer_details::register as register_extraction;
use super::offer_details::prelude::register as register_offer_details;
use super::prepare_offer_imports::register as register_imports;
use super::product_catalog_ids::prelude::register as register_catalog;
use super::structuralize_offer_details::register as register_structuralization;
use crate::platform::{Platform, bus::Bus, db::DbPool, object_storage::ObjectStorage};

pub(crate) fn register(bus: &mut Bus, platform: &Platform) {
    register_token(bus, platform);
    register_catalog(bus, platform);
    register_offer_details(bus, platform);
    register_derived_offer_steps(bus, &platform.db, &platform.object_storage);
}

pub(crate) fn register_derived_offer_steps(bus: &mut Bus, db: &DbPool, storage: &ObjectStorage) {
    register_extraction(bus, storage.clone());
    register_structuralization(bus, storage.clone());
    register_imports(bus, db.clone(), storage.clone());
}
