use super::super::retrieve_sitemap::prelude::SitemapRetrieved;
use super::command::ProcessSitemap;

impl From<SitemapRetrieved> for ProcessSitemap {
    fn from(event: SitemapRetrieved) -> Self {
        Self {
            retrieval_id: event.retrieval_id,
            retailer_code: event.retailer_code,
        }
    }
}
