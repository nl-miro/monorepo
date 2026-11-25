use outbox::record as outbox_record;

use uuid::Uuid;

use super::reacts_to::{Store, StoreError};
use crate::platform::bus::EventContext;
use crate::platform::db::DbPool;
use crate::platform::object_storage::ObjectStorage;
use crate::platform::outbox::{self, OutgoingMessage};
use crate::steps::prelude::{ExportStore, OfferExport};

pub struct PgStore {
    pool: DbPool,
    exports: ExportStore,
}

impl PgStore {
    pub fn new(pool: DbPool, objects: ObjectStorage) -> Self {
        Self {
            exports: ExportStore::new(pool.clone(), objects),
            pool,
        }
    }
}

impl Store for PgStore {
    fn load(&self, id: Uuid) -> Result<Option<OfferExport>, StoreError> {
        Ok(self.exports.load(id)?)
    }

    fn record(
        &self,
        messages: &[OutgoingMessage],
        context: &EventContext,
    ) -> Result<(), StoreError> {
        let mut conn = self.pool.get()?;
        Ok(outbox_record(&mut conn, messages, context)?)
    }
}
