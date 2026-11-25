use crate::steps::prelude::{ExtractOfferPage, OfferPageWasDownloaded};

/// Extracts every offer page as soon as it is downloaded.

impl From<OfferPageWasDownloaded> for ExtractOfferPage {
    fn from(event: OfferPageWasDownloaded) -> Self {
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
