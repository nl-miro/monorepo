//! Database records for saga entities.

use super::schema::shopify_graphql_initial_fills;
use diesel::{Insertable, Queryable, Selectable};
use serde_json::{Error as JsonError, Value as Json};
use thiserror::Error;
use uuid::Uuid;

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = shopify_graphql_initial_fills)]
pub(crate) struct DieselPgSagaRecord {
    #[diesel(serialize_as = Uuid, deserialize_as = Uuid)]
    pub(super) saga_id: Uuid,
    #[diesel(serialize_as = Uuid, deserialize_as = Uuid)]
    pub(super) correlation_id: Uuid,
    pub(super) saga_type: String,
    pub(super) payload: Json,
}

#[derive(Debug, Error)]
#[error("expected saga type {expected}, got {actual}")]
pub(crate) struct SagaTypeMismatch {
    expected: &'static str,
    actual: String,
}

impl SagaTypeMismatch {
    pub(super) fn new(expected: &'static str, actual: String) -> Self {
        Self { expected, actual }
    }
}

#[derive(Debug, Error)]
pub(crate) enum SagaRecordError {
    #[error(transparent)]
    TypeMismatch(#[from] SagaTypeMismatch),
    #[error("invalid saga retailer_code: {0}")]
    InvalidRetailerCode(#[from] JsonError),
}
