//! Shared HTTP request/response contracts and application outcome mapping.
pub mod axum;

use super::super::service::InitializationFailure;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub(crate) struct InitialFillRequest {
    /// Nonempty retailer list. Duplicate retailers are rejected.
    retailers: Vec<String>,
}

impl InitialFillRequest {
    fn retailer_codes(self) -> Result<Vec<RetailerCode>, InitializationFailure> {
        self.retailers
            .into_iter()
            .map(|code| {
                RetailerCode::from_str(&code)
                    .map_err(|_| InitializationFailure::UnknownRetailer(code))
            })
            .collect()
    }
}

#[derive(Serialize, ToSchema)]
pub struct InitialFillError {
    error: String,
}

impl InitialFillError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
        }
    }
}

/// HTTP outcomes rendered by each framework adapter.
pub(crate) enum InitialFillHttpResponse {
    BadRequest(InitialFillError),
    ServiceUnavailable(InitialFillError),
    InternalServerError(InitialFillError),
}

impl From<InitializationFailure> for InitialFillHttpResponse {
    fn from(failure: InitializationFailure) -> Self {
        match failure {
            InitializationFailure::EmptyRetailers
            | InitializationFailure::UnknownRetailer(_)
            | InitializationFailure::DuplicateRetailer(_)
            | InitializationFailure::UnsupportedRetailer(_, _) => {
                Self::BadRequest(InitialFillError::new(failure.to_string()))
            }
            InitializationFailure::QueueFailed(message) => {
                Self::InternalServerError(InitialFillError::new(message))
            }
        }
    }
}
