use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use super::super::schema::{
    processed_sitemaps_data, raw_sitemap_documents_data, sitemaps_processed, sitemaps_raw,
};
use super::entity::{ProcessedSitemap, RawSitemapDocument};
use super::handler::Store;
use super::record::{
    ProcessedSitemapDataPgRecord, ProcessedSitemapPgRecord, RawSitemapDocumentDataPgRecord,
    RawSitemapDocumentPgRecord,
};
use crate::platform::db::{self, DbPool};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("processed sitemap document is too large for storage")]
    DocumentTooLarge,

    #[error("sitemap contains {0} URLs, which exceeds the supported storage maximum")]
    TooManyUrls(usize),

    #[error("invalid stored raw sitemap body size: {0}")]
    InvalidBodySize(i32),
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
    fn raw_documents(&self, retrieval_id: Uuid) -> Result<Vec<RawSitemapDocument>, StoreError> {
        let mut connection = self.pool.get()?;
        let records = sitemaps_raw::table
            .inner_join(raw_sitemap_documents_data::table)
            .filter(sitemaps_raw::retrieval_id.eq(retrieval_id))
            .order(sitemaps_raw::fetched_at.asc())
            .select((
                RawSitemapDocumentPgRecord::as_select(),
                RawSitemapDocumentDataPgRecord::as_select(),
            ))
            .load::<(RawSitemapDocumentPgRecord, RawSitemapDocumentDataPgRecord)>(
                &mut connection,
            )?;

        records
            .into_iter()
            .map(|(record, data)| record.into_entity(data))
            .collect()
    }

    fn store_processed(&self, sitemap: &ProcessedSitemap) -> Result<(), StoreError> {
        let (record, data): (ProcessedSitemapPgRecord, ProcessedSitemapDataPgRecord) =
            sitemap.try_into()?;

        let mut connection = self.pool.get()?;
        connection.transaction(|connection| {
            diesel::insert_into(sitemaps_processed::table)
                .values(record)
                .execute(connection)?;
            diesel::insert_into(processed_sitemaps_data::table)
                .values(data)
                .execute(connection)?;
            Ok(())
        })
    }
}
