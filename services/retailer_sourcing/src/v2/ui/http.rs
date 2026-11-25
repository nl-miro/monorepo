//! HTTP response body for initial-fill requests.
pub(super) mod axum;

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(super) struct InitialFillError {
    error: &'static str,
}
