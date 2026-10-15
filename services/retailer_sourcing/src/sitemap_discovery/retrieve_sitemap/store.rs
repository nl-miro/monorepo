use diesel::prelude::*;
use diesel::r2d2::PoolError;

use super::super::schema::{raw_sitemap_documents_data, sitemaps_raw};
use super::entity::RetrievedSitemapDocument;
use super::handler::Store;
use super::record::{RawSitemapDocumentDataPgRecord, RawSitemapDocumentPgRecord};
use crate::platform::db::{self, DbPool};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),

    #[error("raw sitemap body is too large for storage: {0} bytes")]
    BodyTooLarge(usize),
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
    fn store_documents(&self, documents: &[RetrievedSitemapDocument]) -> Result<(), StoreError> {
        let pairs = documents
            .iter()
            .map(
                <(
                    RawSitemapDocumentPgRecord<'_>,
                    RawSitemapDocumentDataPgRecord<'_>,
                )>::try_from,
            )
            .collect::<Result<Vec<_>, _>>()?;
        let (records, data): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();

        let mut connection = self.pool.get()?;
        connection.transaction(|connection| {
            diesel::insert_into(sitemaps_raw::table)
                .values(&records)
                .execute(connection)?;
            diesel::insert_into(raw_sitemap_documents_data::table)
                .values(&data)
                .execute(connection)?;
            Ok(())
        })
    }
}
