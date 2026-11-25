//! Axum extraction, route registration and response rendering.
use super::{InitialFillError, InitialFillHttpResponse, InitialFillRequest};
use crate::platform::{bus::BusHandle, state::AppState};
use crate::saga::shopify_graphql_initial_fill::service::{InitializationFailure, Service};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::collections::HashSet;
use tracing::error;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(start_initial_fills))
}

#[utoipa::path(post, path="/initial-offer-fills", request_body=InitialFillRequest, responses(
    (status=204, description="Initialization commands queued for all requested retailers"),
    (status=400, body=InitialFillError, description="Empty list, unknown or duplicate retailer"),
    (status=500, body=InitialFillError, description="Failed to queue an initialization command"),
    (status=503, body=InitialFillError, description="Command bus unavailable")
))]
async fn start_initial_fills(
    State(bus): State<Option<BusHandle>>,
    Json(request): Json<InitialFillRequest>,
) -> Result<StatusCode, InitialFillHttpResponse> {
    if request.retailers.is_empty() {
        return Err(InitializationFailure::EmptyRetailers.into());
    }
    let mut seen = HashSet::new();
    for retailer in &request.retailers {
        if !seen.insert(retailer) {
            return Err(InitializationFailure::DuplicateRetailer(retailer.clone()).into());
        }
    }
    run(bus, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn run(
    bus: Option<BusHandle>,
    request: InitialFillRequest,
) -> Result<(), InitialFillHttpResponse> {
    let Some(bus) = bus else {
        let failure = command_bus_unavailable();
        return Err(failure);
    };

    let retailers = request.retailer_codes()?;

    let service = Service::new(bus.command_gateway());

    service.run(retailers).map_err(Into::into)
}

fn command_bus_unavailable() -> InitialFillHttpResponse {
    let message = "command bus is unavailable";
    error!(severity = "critical", "{message}");

    InitialFillHttpResponse::ServiceUnavailable(InitialFillError::new(message))
}

impl IntoResponse for InitialFillHttpResponse {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(body) => (StatusCode::BAD_REQUEST, Json(body)).into_response(),
            Self::ServiceUnavailable(body) => {
                (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
            }
            Self::InternalServerError(body) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde_json::Value;
    use serde_json::from_slice;
    use serde_json::json;

    #[tokio::test]
    async fn queue_failure_returns_the_dispatch_message() {
        let failure = InitializationFailure::QueueFailed("dispatch failed".into());
        let response = InitialFillHttpResponse::from(failure).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            from_slice::<Value>(&body).unwrap(),
            json!({"error": "dispatch failed"})
        );
    }

    #[tokio::test]
    async fn empty_request_returns_400_before_checking_the_bus() {
        let response =
            start_initial_fills(State(None), Json(InitialFillRequest { retailers: vec![] }))
                .await
                .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            from_slice::<Value>(&body).unwrap(),
            json!({"error": "retailers must not be empty"})
        );
    }

    #[tokio::test]
    async fn duplicate_retailer_returns_400_before_checking_the_bus() {
        let request = InitialFillRequest {
            retailers: vec!["ankerau".into(), "ankerca".into(), "ankerau".into()],
        };
        let response = start_initial_fills(State(None), Json(request))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            from_slice::<Value>(&body).unwrap(),
            json!({"error": "duplicate retailer code: ankerau"})
        );
    }

    #[tokio::test]
    async fn unavailable_bus_returns_503_for_nonempty_requests() {
        for retailers in [vec!["unknown".into()], vec!["ankerau".into()]] {
            let response = start_initial_fills(State(None), Json(InitialFillRequest { retailers }))
                .await
                .into_response();
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert_eq!(
                from_slice::<Value>(&body).unwrap(),
                json!({"error": "command bus is unavailable"}),
            );
        }
    }
}
