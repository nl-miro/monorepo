//! Sitemap step registration and events for downstream consumers.

use consolidate_sitemap::prelude::register as consolidate_sitemap_register;
use process_sitemap::prelude::register as process_sitemap_register;
use retrieve_sitemap::prelude::register as retrieve_sitemap_register;

use super::{consolidate_sitemap, process_sitemap, retrieve_sitemap};
use crate::platform::Platform;
use crate::platform::bus::Bus;

pub use super::consolidate_sitemap::prelude::SitemapConsolidated;

/// Registers every sitemap discovery slice.
pub fn register(bus: &mut Bus, platform: &Platform) {
    retrieve_sitemap_register(bus, platform);
    process_sitemap_register(bus, platform);
    consolidate_sitemap_register(bus, platform);
}
