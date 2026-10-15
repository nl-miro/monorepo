//! The consolidated sitemap projection needed to discover offers.
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct ConsolidatedSitemap {
    pub id: Uuid,
    pub product_urls: Vec<String>,
}
