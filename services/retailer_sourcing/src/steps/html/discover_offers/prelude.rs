//! Everything other modules may use from this slice.

use handler::Handler;
use store::PgStore;

use super::{handler, store};
use crate::platform::Platform;
use crate::platform::bus::Bus;

pub use super::command::DiscoverOffers;
pub use super::event::OfferDiscovered;

/// Registers the slice's command handler.
pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(Handler::new(store));
}
