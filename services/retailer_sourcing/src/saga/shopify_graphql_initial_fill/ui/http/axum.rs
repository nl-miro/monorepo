//! Axum extraction, route registration and response rendering.
use super::{
    FailureReason, InitialFillError, InitialFillHttpResponse, InitialFillRequest, RetailerFailure,
};
use crate::platform::{bus::BusHandle, state::AppState};
use crate::saga::shopify_graphql_initial_fill::service::Service;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use retailer_data_extractor::get_storefront_config;
use shared::retailer::{RetailerCode, RetailerCodes, RetailerCodesConversionError};
use std::collections::HashMap;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(start_initial_fills))
}

#[utoipa::path(post, path="/initial-offer-fills", request_body=InitialFillRequest, responses(
    (status=204, description="Initialization commands queued for all requested retailers"),
    (status=400, body=InitialFillError, description="Empty list, duplicate, unknown or GraphQL-unsupported retailers"),
    (status=500, body=InitialFillError, description="Failed to queue an initialization command")
))]
#[rustfmt::skip]
async fn start_initial_fills(State(bus): State<Option<BusHandle>>, Json(request): Json<InitialFillRequest>) -> Result<StatusCode, InitialFillHttpResponse> {

    let RetailerCodes(retailers) = RetailerCodes::try_from(request.retailers)?;

    InitialFillValidationService::validate(&retailers)?;

    let controller = build_controller(bus);
    controller.start(retailers)
}

fn build_controller(bus: Option<BusHandle>) -> InitialFillController {
    let bus = bus.expect("initial fills require a command bus");
    let service = Service::new(bus.command_gateway());
    InitialFillController::new(service)
}

struct InitialFillController {
    service: Service,
}

impl InitialFillController {
    fn new(service: Service) -> Self {
        Self { service }
    }

    fn start(&self, retailers: Vec<RetailerCode>) -> Result<StatusCode, InitialFillHttpResponse> {
        self.service.run(retailers)?;

        Ok(StatusCode::NO_CONTENT)
    }
}

struct InitialFillValidationService;

#[derive(Debug)]
enum InitialFillValidationError {
    EmptyRetailers,
    DuplicateRetailer(RetailerCode),
    UnknownRetailer(String),
    GraphqlUnsupported(RetailerCode),
}

type Res = Result<(), Vec<InitialFillValidationError>>;

impl InitialFillValidationService {
    fn validate(retailers: &[RetailerCode]) -> Res {
        Self::require_retailers(retailers)?;

        Self::check_for_duplicate_retailers(retailers)?;

        Self::check_graphql_support(retailers)?;

        Ok(())
    }

    fn require_retailers(retailers: &[RetailerCode]) -> Res {
        if retailers.is_empty() {
            let errors = vec![InitialFillValidationError::EmptyRetailers];
            return Err(errors);
        }
        Ok(())
    }

    fn check_for_duplicate_retailers(retailers: &[RetailerCode]) -> Res {
        // Report repeated codes once, at their first position in the request.
        let mut counts = HashMap::new();
        for &code in retailers {
            *counts.entry(code).or_insert(0usize) += 1;
        }

        let mut errors = Vec::new();

        for code in retailers {
            if counts.remove(code).is_some_and(|count| count > 1) {
                errors.push(InitialFillValidationError::DuplicateRetailer(*code));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn check_graphql_support(retailers: &[RetailerCode]) -> Res {
        let errors: Vec<_> = retailers
            .iter()
            .copied()
            .filter(|&code| {
                !get_storefront_config(code).is_some_and(|config| config.access.supports_graphql())
            })
            .map(InitialFillValidationError::GraphqlUnsupported)
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl From<RetailerCodesConversionError> for InitialFillHttpResponse {
    fn from(error: RetailerCodesConversionError) -> Self {
        let errors: Vec<_> = error
            .invalid_slugs
            .into_iter()
            .map(InitialFillValidationError::UnknownRetailer)
            .collect();
        errors.into()
    }
}

impl From<Vec<InitialFillValidationError>> for InitialFillHttpResponse {
    fn from(errors: Vec<InitialFillValidationError>) -> Self {
        let mut message = "invalid retailers";
        let mut failures = Vec::new();
        for error in errors {
            let (retailer, reason) = match error {
                InitialFillValidationError::EmptyRetailers => {
                    message = "retailers must not be empty";
                    continue;
                }
                InitialFillValidationError::DuplicateRetailer(code) => {
                    (code.as_str(), FailureReason::Duplicate)
                }
                InitialFillValidationError::UnknownRetailer(code) => (code, FailureReason::Unknown),
                InitialFillValidationError::GraphqlUnsupported(code) => {
                    (code.as_str(), FailureReason::NotGraphql)
                }
            };
            failures.push(RetailerFailure { retailer, reason });
        }
        let body = InitialFillError::validation(message, failures);
        Self::BadRequest(body)
    }
}

impl IntoResponse for InitialFillHttpResponse {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(body) => (StatusCode::BAD_REQUEST, Json(body)).into_response(),
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
    use serde_json::{Value, from_slice, json};

    async fn check_bad_request(retailers: &[&str], expected: Value) {
        let request = InitialFillRequest {
            retailers: retailers.iter().map(|code| (*code).to_owned()).collect(),
        };
        let response = start_initial_fills(State(None), Json(request))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(from_slice::<Value>(&bytes).unwrap(), expected);
    }

    #[tokio::test]
    #[should_panic(expected = "initial fills require a command bus")]
    async fn valid_request_without_a_bus_panics() {
        let request = InitialFillRequest {
            retailers: vec!["ankerau".into()],
        };
        let _ = start_initial_fills(State(None), Json(request)).await;
    }

    #[tokio::test]
    async fn empty_list_is_a_request_level_failure() {
        check_bad_request(
            &[],
            json!({"error":"retailers must not be empty","failures":[]}),
        )
        .await;
    }

    #[tokio::test]
    async fn duplicate_codes_are_reported_once_before_graphql_checks() {
        check_bad_request(
            &["ankerkr", "ankerau", "ankerau", "ankerkr", "ankerkr"],
            json!({"error":"invalid retailers", "failures":[
                {"retailer":"ankerkr","reason":"duplicate"},
                {"retailer":"ankerau","reason":"duplicate"}
            ]}),
        )
        .await;
    }

    #[tokio::test]
    async fn unknown_codes_are_reported_before_duplicate_and_graphql_checks() {
        check_bad_request(
            &["unknown", "ankerkr", "AnkerAu", "ankerkr"],
            json!({"error":"invalid retailers", "failures":[
                {"retailer":"unknown","reason":"unknown"},
                {"retailer":"AnkerAu","reason":"unknown"}
            ]}),
        )
        .await;
    }

    #[tokio::test]
    async fn all_graphql_unsupported_retailers_are_reported() {
        check_bad_request(
            &["ankerkr", "ankerau", "micom"],
            json!({"error":"invalid retailers", "failures":[
                {"retailer":"ankerkr","reason":"not_graphql"},
                {"retailer":"micom","reason":"not_graphql"}
            ]}),
        )
        .await;
    }

    #[test]
    fn valid_codes_keep_their_request_order() {
        let RetailerCodes(retailers) =
            RetailerCodes::try_from(vec!["ankerca".into(), "ankerau".into()]).unwrap();
        assert_eq!(retailers, [RetailerCode::AnkerCa, RetailerCode::AnkerAu]);
        InitialFillValidationService::validate(&retailers).unwrap();
    }
}
