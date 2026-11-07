//! Repository operations for export processing.

use diesel::prelude::*;
use diesel::r2d2::PoolError;
use diesel::sql_types::Uuid as SqlUuid;
use uuid::Uuid;

use super::entity::OfferExport;
use super::store::{OfferExportPgRecord, StoreError};
use crate::platform::db::{self, DbPool};
use crate::platform::object_storage::{OFFER_EXPORT_BUCKET, ObjectStorageRepository};

#[derive(Debug, thiserror::Error)]
pub(crate) enum OfferExportRepositoryError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),
    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),
    #[error("more than one completed offer export record with ID {0}")]
    DuplicateId(Uuid),
    #[error(transparent)]
    Snapshot(#[from] StoreError),
}

pub(crate) trait OfferExportRepository: Send + Sync + 'static {
    fn load(&self, id: Uuid) -> Result<Option<OfferExport>, OfferExportRepositoryError>;
}

pub(crate) struct PgOfferExportRepository {
    pool: DbPool,
    object_repository: ObjectStorageRepository,
}

impl PgOfferExportRepository {
    pub fn new(pool: DbPool, object_repository: ObjectStorageRepository) -> Self {
        // Retain both dependencies for export loading.

        Self {
            pool,
            object_repository,
        }
    }
}

impl OfferExportRepository for PgOfferExportRepository {
    fn load(&self, id: Uuid) -> Result<Option<OfferExport>, OfferExportRepositoryError> {
        let sql = "SELECT * FROM offer_export WHERE id = $1 AND status = 'completed' LIMIT 2";
        let mut records = diesel::sql_query(sql)
            .bind::<SqlUuid, _>(id)
            .load::<OfferExportPgRecord>(&mut self.pool.get()?)?;

        let record = match records.len() {
            0 => return Ok(None),
            1 => records.remove(0),
            _ => return Err(OfferExportRepositoryError::DuplicateId(id)),
        };

        let storage = record.storage.as_ref().ok_or(StoreError::InvalidRecord)?;
        if record.prepared_body.is_some() {
            return Err(StoreError::InvalidRecord.into());
        }

        let body = self
            .object_repository
            .get(OFFER_EXPORT_BUCKET, record.id, &storage.0)
            .map_err(StoreError::from)?;

        let prepared = record.into_prepared(body)?;

        Ok(Some(prepared.snapshot))
    }
}
