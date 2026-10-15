//! HTTP entry point: lists the fetched sitemap files, a page at a time.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable, Text, Uuid as SqlUuid};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::platform::db::DbPool;

/// The most sources one page may hold. Each carries a whole sitemap file.
const MAX_LIMIT: u32 = 100;

/// Each fetched sitemap file with its URL, crawl, body and the retailer of the
/// `RetrieveSitemap` command that fetched it, newest first, skipping `$2` and
/// returning up to `$1`.
const SOURCES_QUERY: &str = "
    SELECT r.id, r.retrieval_id, r.url, d.body, (
        SELECT c.payload::jsonb ->> 'retailer_code'
        FROM command_entries c
        WHERE c.command_type = 'RetrieveSitemap'
            AND c.payload::jsonb ->> 'retrieval_id' = r.retrieval_id::text
        LIMIT 1
    ) AS retailer_code
    FROM sitemaps_raw r
    JOIN raw_sitemap_documents_data d ON d.id = r.id
    ORDER BY r.fetched_at DESC, r.id
    LIMIT $1 OFFSET $2";

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Page {
    /// How many sources to return, from 1 to 100.
    #[serde(default = "default_limit")]
    #[param(default = 20, minimum = 1, maximum = 100)]
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
pub struct SourcesResponse {
    sources: Vec<Source>,
}

#[derive(Serialize, ToSchema, QueryableByName)]
pub struct Source {
    #[diesel(sql_type = SqlUuid)]
    #[schema(value_type = String, format = "uuid")]
    id: Uuid,
    /// The crawl that fetched the file.
    #[diesel(sql_type = SqlUuid)]
    #[schema(value_type = String, format = "uuid")]
    retrieval_id: Uuid,
    /// The URL the file was fetched from.
    #[diesel(sql_type = Text)]
    url: String,
    /// `null` when the command that fetched the file is gone.
    #[diesel(sql_type = Nullable<Text>)]
    retailer_code: Option<String>,
    /// The sitemap file as fetched.
    #[diesel(sql_type = Text)]
    body: String,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    error: String,
}

/// Lists the fetched sitemap files, newest first.
#[utoipa::path(
    get,
    path = "/development/sitemaps/sources",
    params(Page),
    security(("bearer" = [])),
    responses(
        (status = 200, description = "A page of sources", body = SourcesResponse),
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
        Ok(Ok(sources)) => Json(SourcesResponse { sources }).into_response(),
        Ok(Err(message)) => error(StatusCode::INTERNAL_SERVER_ERROR, message),
        Err(join) => error(StatusCode::INTERNAL_SERVER_ERROR, join.to_string()),
    }
}

fn load_sources(db: &DbPool, page: &Page) -> Result<Vec<Source>, String> {
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
