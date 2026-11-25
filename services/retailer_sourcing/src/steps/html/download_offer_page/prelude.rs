//! Everything other modules may use from this slice.

use handler::Handler;
use store::PgStore;

use super::{handler, reacts_to, store};
use crate::platform::Platform;
use crate::platform::bus::Bus;

pub use super::command::DownloadOfferPage;
pub use super::event::OfferPageWasDownloaded;

/// Registers the slice's command handler.
pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(Handler::new(store, platform.offer_http.clone()));
}
