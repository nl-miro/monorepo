//! Shared record for persisted retailer sagas.

use serde::{Serialize, de::DeserializeOwned};
use shared::retailer::RetailerCode;
use std::fmt::Debug;
use uuid::Uuid;

/// A saga's state is stored as a JSON object under its stable type name.
pub(crate) trait Saga: Debug + Serialize + DeserializeOwned + 'static {
    const TYPE: &'static str;
}

#[derive(Debug)]
pub(crate) struct SagaRecord<S: Saga> {
    pub(crate) id: Uuid,
    pub(crate) correlation_id: Uuid,
    pub(crate) status: String,
    pub(crate) retailer_code: RetailerCode,
    pub(crate) payload: S,
}
