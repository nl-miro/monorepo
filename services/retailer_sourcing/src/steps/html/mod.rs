//! Discovers offers in consolidated sitemaps and downloads their pages.

mod discover_offers;
mod download_offer_page;

use crate::platform::Platform;
use crate::platform::bus::Bus;
use discover_offers::prelude::register as register_discover_offers;
use download_offer_page::prelude::register as register_download_offer_page;

pub(crate) use discover_offers::prelude::{DiscoverOffers, OfferDiscovered};
pub(crate) use download_offer_page::prelude::{DownloadOfferPage, OfferPageWasDownloaded};
pub(crate) use download_offer_page::schema::offer_details_raw;

pub(crate) fn register_html_steps(bus: &mut Bus, platform: &Platform) {
    register_discover_offers(bus, platform);
    register_download_offer_page(bus, platform);
}
