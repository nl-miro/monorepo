//! Public entry point for workflow steps.

pub(crate) use super::graphql::acquire_storefront_token::{
    AcquireStorefrontToken, StorefrontTokenAcquired,
};
pub(crate) use super::graphql::extract_offer_details::{
    ExtractOfferDetails, OfferDetailsExtracted,
};
pub(crate) use super::graphql::offer_details::OfferDetailsFetched;
pub(crate) use super::graphql::offer_details::{FetchOfferDetails, SourceType};
pub(crate) use super::graphql::product_catalog_ids::{
    FetchProductCatalogIdentifiers, ProductCatalogIdsRetrieved,
};
pub(crate) use super::graphql::request_initial_fill::{
    InitialFillRequested, RequestInitialFill, register as register_initial_fill,
};
pub(crate) use super::graphql::structuralize_offer_details::StructuralizeOfferDetails;
pub(crate) use super::html::OfferDiscovered;
pub(crate) use super::html::{
    DiscoverOffers, DownloadOfferPage, OfferPageWasDownloaded, offer_details_raw,
    register_html_steps,
};
#[cfg(test)]
pub(crate) use super::processing_offers::prelude::export_testing;
pub(crate) use super::processing_offers::prelude::shopify_imports;
pub(crate) use super::processing_offers::prelude::{
    ExportStore, ExportStoreError, OfferExport, register as register_offer_processing_steps,
};
pub use super::processing_offers::prelude::{ExtractOfferPage, OfferPageWasExtracted};
pub use super::processing_offers::prelude::{NormalizeOffer, OfferWasNormalized};
pub use super::processing_offers::prelude::{OfferExportWasPrepared, ProcessOfferForExport};
pub use super::processing_offers::prelude::{OfferWasStructuralized, StructuralizeOffer};

pub(crate) use super::register_shopify_steps;
pub use super::sitemap::prelude::ProcessSitemap;
pub use super::sitemap::prelude::RetrieveSitemap;
pub use super::sitemap::prelude::{ConsolidateSitemap, SitemapConsolidated};
pub(crate) use super::sitemap::prelude::{SitemapProcessed, SitemapRetrieved};
pub(crate) use super::sitemap::prelude::{
    register as register_sitemap_steps, routes as sitemap_routes,
};
