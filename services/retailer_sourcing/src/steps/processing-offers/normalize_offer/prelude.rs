//! Everything other modules may use from this slice.

use handler::Handler;
use store::PgStore;

use super::{handler, store};
use crate::platform::Platform;
use crate::platform::bus::Bus;
use crate::steps::processing_offers::Retailers;

pub use super::command::NormalizeOffer;
pub use super::event::OfferWasNormalized;

/// Registers the slice's command handler.
pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(Handler::new(store, Retailers));
}
