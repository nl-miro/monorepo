use bodul_shared::retailer::RetailerCodeConversionError;
use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use super::super::schema::{
    consolidated_sitemaps_data, processed_sitemaps_data, sitemaps_consolidated, sitemaps_processed,
};
use super::entity::{ConsolidatedSitemap, ProcessedSitemap};
use super::handler::Store;
use super::record::{
    ConsolidatedSitemapDataPgRecord, ConsolidatedSitemapPgRecord, ProcessedSitemapDataPgRecord,
    ProcessedSitemapPgRecord,
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

    #[error("invalid stored retailer code: {0}")]
    InvalidRetailerCode(#[from] RetailerCodeConversionError),

    #[error("{0}")]
    Unexpected(String),
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
    fn processed_sitemap(&self, id: Uuid) -> Result<ProcessedSitemap, StoreError> {
        let mut connection = self.pool.get()?;
        let (record, document) = sitemaps_processed::table
            .inner_join(processed_sitemaps_data::table)
            .filter(sitemaps_processed::id.eq(id))
            .select((
                ProcessedSitemapPgRecord::as_select(),
                ProcessedSitemapDataPgRecord::as_select(),
            ))
            .get_result::<(ProcessedSitemapPgRecord, ProcessedSitemapDataPgRecord)>(
                &mut connection,
            )?;

        (record, document).try_into()
    }

    fn store_consolidated(&self, consolidated: &ConsolidatedSitemap) -> Result<(), StoreError> {
        let (record, data): (ConsolidatedSitemapPgRecord, ConsolidatedSitemapDataPgRecord) =
            consolidated.try_into()?;

        let mut connection = self.pool.get()?;
        connection.transaction(|connection| {
            diesel::insert_into(sitemaps_consolidated::table)
                .values(record)
                .execute(connection)?;
            diesel::insert_into(consolidated_sitemaps_data::table)
                .values(data)
                .execute(connection)?;
            Ok(())
        })
    }
}
