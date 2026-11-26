//! Persistence operations for saga entities.

use diesel::prelude::*;
use diesel::r2d2::PoolError;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{insert_into, update};
use serde_json::Error as JsonError;
use thiserror::Error;
use uuid::Uuid;

use super::RecordableSaga;
use super::record::{DieselPgSagaRecord, SagaRecordError, SagaTypeMismatch};
use super::schema::shopify_graphql_initial_fills::{correlation_id, payload, saga_type, table};
use crate::platform::db::DbPool;

#[derive(Debug, Error)]
pub(crate) enum RepositoryError {
    #[error(transparent)]
    Connection(#[from] PoolError),
    #[error(transparent)]
    Database(#[from] DieselError),
    #[error(transparent)]
    SagaTypeMismatch(#[from] SagaTypeMismatch),
    #[error("invalid saga retailer_code: {0}")]
    InvalidRetailerCode(#[source] JsonError),
}

impl RepositoryError {
    pub(crate) fn is_duplicate(&self) -> bool {
        matches!(
            self,
            Self::Database(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            ))
        )
    }
}

pub(crate) trait SagaRepository {
    /// Returns a database not-found error if the saga has not been saved.
    fn load<S>(&self, id: Uuid) -> Result<S, RepositoryError>
    where
        S: RecordableSaga + TryFrom<DieselPgSagaRecord, Error = SagaRecordError>;
    /// Inserts a new saga, returning a database error for a duplicate ID.
    fn insert<S>(&self, saga: S) -> Result<(), RepositoryError>
    where
        S: RecordableSaga + TryInto<DieselPgSagaRecord, Error = SagaTypeMismatch>;
    /// Updates an existing saga of the same type, or returns a database not-found error.
    fn update<S>(&self, saga: S) -> Result<(), RepositoryError>
    where
        S: RecordableSaga + TryInto<DieselPgSagaRecord, Error = SagaTypeMismatch>;
}

pub(crate) struct DieselPgSagaRepository {
    pool: DbPool,
}

impl DieselPgSagaRepository {
    pub(crate) fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl SagaRepository for DieselPgSagaRepository {
    fn load<S>(&self, id: Uuid) -> Result<S, RepositoryError>
    where
        S: RecordableSaga + TryFrom<DieselPgSagaRecord, Error = SagaRecordError>,
    {
        let record: DieselPgSagaRecord = table
            .find(id)
            .select(DieselPgSagaRecord::as_select())
            .first(&mut self.pool.get()?)?;
        S::try_from(record).map_err(|error| match error {
            SagaRecordError::TypeMismatch(error) => RepositoryError::SagaTypeMismatch(error),
            SagaRecordError::InvalidRetailerCode(error) => {
                RepositoryError::InvalidRetailerCode(error)
            }
        })
    }

    fn insert<S>(&self, saga: S) -> Result<(), RepositoryError>
    where
        S: RecordableSaga + TryInto<DieselPgSagaRecord, Error = SagaTypeMismatch>,
    {
        let record: DieselPgSagaRecord = saga.try_into()?;
        insert_into(table)
            .values(record)
            .execute(&mut self.pool.get()?)?;
        Ok(())
    }

    fn update<S>(&self, saga: S) -> Result<(), RepositoryError>
    where
        S: RecordableSaga + TryInto<DieselPgSagaRecord, Error = SagaTypeMismatch>,
    {
        let record: DieselPgSagaRecord = saga.try_into()?;
        let affected = update(
            table
                .find(Uuid::from(record.saga_id))
                .filter(saga_type.eq(record.saga_type)),
        )
        .set((
            correlation_id.eq(record.correlation_id),
            payload.eq(record.payload),
        ))
        .execute(&mut self.pool.get()?)?;
        if affected == 0 {
            return Err(DieselError::NotFound.into());
        }
        Ok(())
    }
}
