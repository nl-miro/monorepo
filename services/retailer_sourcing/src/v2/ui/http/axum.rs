//! Axum extraction, route registration and response rendering.
use super::InitialFillError;
use crate::platform::{bus::BusHandle, state::AppState};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::saga::prelude::InitialFillService;

#[utoipa::path(
    post, path = "/initial-fill", request_body = Vec<String>,
    responses(
        (status = 202, description = "Initial-fill request accepted"),
        (status = 422, description = "Expected an array of retailer code strings"),
        (status = 503, body = InitialFillError, description = "Command bus unavailable")
    )
)]
async fn initial_fill(
    State(bus): State<Option<BusHandle>>,
    Json(retailer_codes): Json<Vec<String>>,
) -> Response {
    let Some(bus) = bus else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(InitialFillError {
                error: "command bus is unavailable",
            }),
        )
            .into_response();
    };

    let service = InitialFillService::new(bus.dispatcher().clone());
    service.run(retailer_codes).await;
    StatusCode::ACCEPTED.into_response()
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(initial_fill))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde_json::Value;
    use serde_json::from_slice;
    use serde_json::{json, to_value};
    use utoipa::OpenApi;

    #[test]
    fn documents_the_initial_fill_contract() {
        #[derive(OpenApi)]
        #[openapi(paths(initial_fill))]
        struct Api;

        let api = Api::openapi();
        let operation = api.paths.paths["/initial-fill"].post.as_ref().unwrap();
        let operation = to_value(operation).unwrap();
        let schema = &operation["requestBody"]["content"]["application/json"]["schema"];
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");
        let responses = operation["responses"].as_object().unwrap();
        assert_eq!(responses.len(), 3);
        assert!(responses["202"].get("content").is_none());
        assert!(responses["422"].is_object());
        assert!(responses["503"]["content"]["application/json"].is_object());
    }

    #[tokio::test]
    async fn unavailable_bus_returns_503_before_retailer_validation() {
        for retailers in [vec![], vec!["unknown".into()], vec!["ankerau".into()]] {
            let response = initial_fill(State(None), Json(retailers)).await;
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert_eq!(
                from_slice::<Value>(&body).unwrap(),
                json!({"error": "command bus is unavailable"}),
            );
        }
    }
}
