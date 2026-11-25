//! Everything other modules may use from this slice.

use super::reacts_to::on_offer_was_normalized;
use handler::Handler;
use store::PgStore;

use super::{handler, store};
use crate::platform::Platform;
use crate::platform::bus::Bus;
use crate::steps::html::Retailers;

pub use super::event::OfferWasStructuralized;

/// Registers the slice's handler and the reaction that starts it.
pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(Handler::new(store, Retailers));
    bus.on(on_offer_was_normalized);
}
