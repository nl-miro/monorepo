use super::listener::Listener;
use super::outbox_repository::PgOutboxRepository;
use crate::offer_discovery::prelude::PgOfferExportRepository;
use crate::platform::Platform;
use crate::platform::bus::Bus;

pub fn register(bus: &mut Bus, platform: &Platform) {
    let exports =
        PgOfferExportRepository::new(platform.db.clone(), platform.object_storage.clone());
    let outbox = PgOutboxRepository::new(platform.db.clone());

    bus.listen(Listener::new(exports, outbox));
}
