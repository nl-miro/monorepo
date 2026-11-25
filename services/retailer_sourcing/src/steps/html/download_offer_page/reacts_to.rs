use super::super::discover_offers::prelude::OfferDiscovered;
use super::command::DownloadOfferPage;

/// Downloads every offer's page as soon as the offer is discovered.

impl From<OfferDiscovered> for DownloadOfferPage {
    fn from(event: OfferDiscovered) -> Self {
        Self {
            offer_id: event.offer_id,
            consolidated_sitemap_id: event.consolidated_sitemap_id,
            retailer_code: event.retailer_code,
            url: event.url,
        }
    }
}
