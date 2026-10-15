//! HTTP entry point: lists the downloaded offer pages, a page at a time.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Text, Uuid as SqlUuid};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::platform::db::DbPool;

/// The most sources one page may hold. Each carries a whole offer page.
const MAX_LIMIT: u32 = 1000;

/// Each downloaded offer page with its offer, URL, retailer and body, newest
/// first, skipping `$2` and returning up to `$1`.
const SOURCES_QUERY: &str = "
    SELECT r.id, r.offer_id, r.url, r.retailer_code, d.body
    FROM offer_details_raw r
    JOIN raw_offers_data d ON d.id = r.id
    ORDER BY r.fetched_at DESC, r.id
    LIMIT $1 OFFSET $2";

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Page {
    /// How many sources to return, from 1 to 1000.
    #[serde(default = "default_limit")]
    #[param(default = 20, minimum = 1, maximum = 1000)]
    limit: u32,
    /// How many sources to skip.
    #[serde(default)]
    #[param(default = 0)]
    offset: u32,
}

fn default_limit() -> u32 {
    20
}

#[derive(Serialize, ToSchema)]
pub struct OfferSourcesResponse {
    sources: Vec<OfferSource>,
}

#[derive(Serialize, ToSchema, QueryableByName)]
pub struct OfferSource {
    #[diesel(sql_type = SqlUuid)]
    #[schema(value_type = String, format = "uuid")]
    id: Uuid,
    /// The offer the page belongs to.
    #[diesel(sql_type = SqlUuid)]
    #[schema(value_type = String, format = "uuid")]
    offer_id: Uuid,
    /// The URL the page was downloaded from.
    #[diesel(sql_type = Text)]
    url: String,
    /// The retailer the offer belongs to.
    #[diesel(sql_type = Text)]
    retailer_code: String,
    /// The offer page's HTML as downloaded.
    #[diesel(sql_type = Text)]
    body: String,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    error: String,
}

/// Lists the downloaded offer pages, newest first.
#[utoipa::path(
    get,
    path = "/development/offer-details/sources",
    params(Page),
    security(("bearer" = [])),
    responses(
        (status = 200, description = "A page of sources", body = OfferSourcesResponse),
        (status = 400, description = "Limit out of range", body = ErrorBody),
        (status = 401, description = "Missing or wrong bearer token", body = ErrorBody),
        (status = 500, description = "The sources couldn't be read", body = ErrorBody),
        (status = 503, description = "No database (no DATABASE_URL)", body = ErrorBody),
    )
)]
pub async fn list_sources(State(db): State<Option<DbPool>>, Query(page): Query<Page>) -> Response {
    if !(1..=MAX_LIMIT).contains(&page.limit) {
        let message = format!("limit must be between 1 and {MAX_LIMIT}");
        return error(StatusCode::BAD_REQUEST, message);
    }
    let Some(db) = db else {
        let message = "there is no database; start the service with DATABASE_URL";
        return error(StatusCode::SERVICE_UNAVAILABLE, message.to_string());
    };

    // Diesel's connections block, so query off the async runtime.
    match tokio::task::spawn_blocking(move || load_sources(&db, &page)).await {
        Ok(Ok(sources)) => Json(OfferSourcesResponse { sources }).into_response(),
        Ok(Err(message)) => error(StatusCode::INTERNAL_SERVER_ERROR, message),
        Err(join) => error(StatusCode::INTERNAL_SERVER_ERROR, join.to_string()),
    }
}

fn load_sources(db: &DbPool, page: &Page) -> Result<Vec<OfferSource>, String> {
    let mut connection = db.get().map_err(|error| error.to_string())?;
    diesel::sql_query(SOURCES_QUERY)
        .bind::<BigInt, _>(i64::from(page.limit))
        .bind::<BigInt, _>(i64::from(page.offset))
        .load(&mut connection)
        .map_err(|error| error.to_string())
}

fn error(status: StatusCode, error: String) -> Response {
    (status, Json(ErrorBody { error })).into_response()
}
