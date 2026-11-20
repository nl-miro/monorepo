//! Everything other modules may use from this slice.

use super::reacts_to::on_offer_discovered;
use handler::Handler;
use store::PgStore;

use super::{handler, store};
use crate::platform::Platform;
use crate::platform::bus::Bus;

pub use super::event::OfferPageWasDownloaded;

/// Registers the slice's handler and the reaction that starts it.
pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(Handler::new(store, platform.offer_http.clone()));
    bus.on(on_offer_discovered);
}
