//! Shared HTTP request/response contracts and application outcome mapping.
pub mod axum;

use super::super::service::InitializationFailure;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub(crate) struct InitialFillRequest {
    /// Nonempty retailer list. Duplicate retailers are rejected.
    retailers: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InitialFillError {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    failures: Option<Vec<RetailerFailure>>,
}

#[derive(Debug, Serialize, ToSchema)]
struct RetailerFailure {
    retailer: String,
    reason: FailureReason,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum FailureReason {
    Duplicate,
    Unknown,
    NotGraphql,
}

impl InitialFillError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            failures: None,
        }
    }

    fn validation(message: impl Into<String>, failures: Vec<RetailerFailure>) -> Self {
        Self {
            error: message.into(),
            failures: Some(failures),
        }
    }
}

/// HTTP outcomes rendered by each framework adapter.
pub(crate) enum InitialFillHttpResponse {
    BadRequest(InitialFillError),
    InternalServerError(InitialFillError),
}

impl From<InitializationFailure> for InitialFillHttpResponse {
    fn from(failure: InitializationFailure) -> Self {
        match failure {
            InitializationFailure::QueueFailed(message) => {
                Self::InternalServerError(InitialFillError::new(message))
            }
        }
    }
}
