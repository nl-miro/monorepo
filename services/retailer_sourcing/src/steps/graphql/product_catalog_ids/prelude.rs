use super::Handler;
use crate::platform::{Platform, bus::Bus};

pub(crate) use super::FetchProductCatalogIdentifiers;

pub fn register(bus: &mut Bus, platform: &Platform) {
    bus.handle(Handler(platform.offer_http.clone()));
}
