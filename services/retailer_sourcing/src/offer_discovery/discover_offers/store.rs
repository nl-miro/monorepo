use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use super::entity::ConsolidatedSitemap;
use super::handler::Store;
use super::record::{ConsolidatedSitemapPgRecord, consolidated_sitemaps_data};
use crate::platform::db::{self, DbPool};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),

    #[error("unreadable consolidated sitemap content: {0}")]
    Content(#[from] serde_json::Error),
}

/// Postgres storage for this slice.
pub struct PgStore {
    pool: DbPool,
}

impl PgStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Store for PgStore {
    fn consolidated_sitemap(&self, id: Uuid) -> Result<ConsolidatedSitemap, StoreError> {
        let mut connection = self.pool.get()?;
        let record = consolidated_sitemaps_data::table
            .find(id)
            .select(ConsolidatedSitemapPgRecord::as_select())
            .get_result::<ConsolidatedSitemapPgRecord>(&mut connection)?;
        record.try_into()
    }
}
