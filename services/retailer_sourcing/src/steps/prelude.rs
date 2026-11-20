//! Workflow step registration and contracts used by the saga and downstream steps.

pub(crate) use super::graphql::{
    acquire_storefront_token::AcquireStorefrontToken,
    initialize_retailer_fill::{InitializeRetailerFill, RetailerFillInitialized},
    prelude::register as register_graphql,
    product_catalog_ids::prelude::FetchProductCatalogIdentifiers,
};

#[cfg(test)]
pub(crate) use super::html::prelude::export_testing;
pub(crate) use super::html::prelude::{
    ExportStore, ExportStoreError, OfferExport, OfferExportWasPrepared, register as register_html,
    shopify_imports,
};
pub(crate) use super::record_export_messages::prelude::register as register_record_export_messages;
pub(crate) use super::sitemap::prelude::{SitemapConsolidated, register as register_sitemap};

#[cfg(test)]
pub(crate) use super::graphql::{
    acquire_storefront_token, offer_details, product_catalog_ids, storefront_engine,
};

#[cfg(test)]
pub(crate) use super::graphql::prelude::{register_derived_offer_steps, register_initialization};
