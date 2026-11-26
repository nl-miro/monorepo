//! Shared commands, events, export types, and registration for offer processing.

use extract_offer_page::prelude::register as extract_offer_page_register;
use normalize_offer::prelude::register as normalize_offer_register;
use processing_for_export::prelude::register as processing_for_export_register;
use record_export_messages::prelude::register as record_export_messages_register;
use structuralize_offer::prelude::register as structuralize_offer_register;

use super::{
    extract_offer_page, normalize_offer, processing_for_export, record_export_messages,
    structuralize_offer,
};
use crate::platform::Platform;
use crate::platform::bus::Bus;

pub use super::extract_offer_page::prelude::{ExtractOfferPage, OfferPageWasExtracted};
pub use super::normalize_offer::prelude::{NormalizeOffer, OfferWasNormalized};
pub use super::processing_for_export::prelude::{OfferExportWasPrepared, ProcessOfferForExport};
pub use super::structuralize_offer::prelude::{OfferWasStructuralized, StructuralizeOffer};

pub(crate) use super::processing_for_export::prelude::shopify_imports;
#[cfg(test)]
pub(crate) use super::processing_for_export::prelude::testing as export_testing;
pub use super::processing_for_export::prelude::{
    OfferExport, PgStore as ExportStore, StoreError as ExportStoreError,
};

/// Registers every offer processing slice.
pub fn register(bus: &mut Bus, platform: &Platform) {
    extract_offer_page_register(bus, platform);
    normalize_offer_register(bus, platform);
    structuralize_offer_register(bus, platform);
    processing_for_export_register(bus, platform);
    record_export_messages_register(bus, platform);
}
