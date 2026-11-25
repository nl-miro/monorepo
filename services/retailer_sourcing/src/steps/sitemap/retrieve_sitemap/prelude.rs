//! Everything other modules may use from this slice.

use handler::Handler;
use store::PgStore;

use super::super::Retailers;
use super::{handler, store};
use crate::platform::Platform;
use crate::platform::bus::Bus;

pub use super::event::SitemapRetrieved;

/// Registers the slice's handler.
pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(Handler::new(store, platform.http.clone(), Retailers));
}
