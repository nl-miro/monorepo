use super::super::process_sitemap::prelude::SitemapProcessed;
use super::command::ConsolidateSitemap;

/// Consolidates every sitemap as soon as it has been processed.
impl From<SitemapProcessed> for ConsolidateSitemap {
    fn from(event: SitemapProcessed) -> Self {
        Self {
            processed_sitemap_id: event.processed_sitemap_id,
        }
    }
}
