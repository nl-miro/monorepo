use super::command::DiscoverOffers;
use crate::steps::prelude::SitemapConsolidated;

/// Discovers offers in every consolidated sitemap that lists products.

impl From<SitemapConsolidated> for DiscoverOffers {
    fn from(event: SitemapConsolidated) -> Self {
        Self {
            consolidated_sitemap_id: event.consolidated_sitemap_id,
            retailer_code: event.retailer_code,
        }
    }
}
