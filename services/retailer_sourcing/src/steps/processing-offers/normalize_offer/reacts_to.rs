use crate::steps::prelude::{NormalizeOffer, OfferPageWasExtracted};

/// Normalizes every offer as soon as its page is extracted.

impl From<OfferPageWasExtracted> for NormalizeOffer {
    fn from(event: OfferPageWasExtracted) -> Self {
        Self {
            storage: event.storage,
            scraped_at: event.scraped_at,
            offer_id: event.offer_id,
            consolidated_sitemap_id: event.consolidated_sitemap_id,
            retailer_code: event.retailer_code,
            url: event.url,
        }
    }
}
