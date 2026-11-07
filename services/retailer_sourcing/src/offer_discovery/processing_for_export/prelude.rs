pub(crate) use super::repository::{
    OfferExportRepository, OfferExportRepositoryError, PgOfferExportRepository,
};
use super::{handler, reacts_to, store};
use crate::platform::{Platform, bus::Bus};

#[cfg(test)]
pub use super::entity::OfferExport;
pub use super::event::OfferExportWasPrepared;

pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = store::PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(handler::Handler::new(store));
    bus.on(reacts_to::on);
}

/// Builds exports for other contexts' tests.
#[cfg(test)]
pub(crate) mod testing {
    pub(crate) use super::super::conversion::tests::{ANKER, source};
    pub use super::super::entity::{
        ContentNextGeneration, ExclusionReason, ExportEntry, ExportExclusion, ExportSource,
        OfferImport,
    };
    use super::super::handler::Store;
    use super::super::store::PgStore;
    pub(crate) use super::super::test_facts::{
        assert_payloads as assert_fact_payloads, document as fact_document,
    };
    use super::{OfferExport, OfferExportWasPrepared};
    use crate::platform::db::DbPool;
    use crate::platform::object_storage::{ObjectStorageRepository, STRUCTURALIZED_OFFERS_BUCKET};

    /// A source whose structuralized `document` is in object storage.
    pub fn stored_source(objects: &ObjectStorageRepository, document: &str) -> ExportSource {
        let mut source = source();
        let scraped_at = source.storage.scraped_at;
        source.storage = objects
            .create_offer(STRUCTURALIZED_OFFERS_BUCKET, document)
            .unwrap();
        source.storage.scraped_at = scraped_at;

        source
    }

    /// Prepares and completes the export of `source`.
    pub fn completed(
        db: &DbPool,
        objects: ObjectStorageRepository,
        source: ExportSource,
    ) -> OfferExport {
        let store = PgStore::new(db.clone(), objects);
        store.complete(store.prepare(source).unwrap()).unwrap()
    }

    /// Prepares the export of `source` without completing it.
    pub fn pending(
        db: &DbPool,
        objects: ObjectStorageRepository,
        source: ExportSource,
    ) -> OfferExport {
        PgStore::new(db.clone(), objects)
            .prepare(source)
            .unwrap()
            .snapshot
    }

    pub fn event(export: &OfferExport) -> OfferExportWasPrepared {
        OfferExportWasPrepared {
            export_id: export.id,
            source: export.source.clone(),
        }
    }
}
