use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use super::super::schema::{
    normalized_offers_data, offer_details_normalized, offer_details_structuralized,
    structuralized_offers_data,
};
use super::entity::{NormalizedOffer, StructuralizedOffer};
use super::handler::Store;
use super::record::{
    NormalizedOfferPgRecord, StructuralizedOfferDataPgRecord, StructuralizedOfferPgRecord,
};
use crate::platform::db::{self, DbPool};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),
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
    fn normalized_offer(&self, offer_id: Uuid) -> Result<Option<NormalizedOffer>, StoreError> {
        let mut connection = self.pool.get()?;
        let record = offer_details_normalized::table
            .inner_join(normalized_offers_data::table)
            .filter(offer_details_normalized::offer_id.eq(offer_id))
            .select((
                offer_details_normalized::offer_id,
                normalized_offers_data::document,
            ))
            .first::<NormalizedOfferPgRecord>(&mut connection)
            .optional()?;
        Ok(record.map(Into::into))
    }

    fn store_structuralized_offer(&self, entity: &StructuralizedOffer) -> Result<(), StoreError> {
        let (record, mut data): (
            StructuralizedOfferPgRecord,
            StructuralizedOfferDataPgRecord<'_>,
        ) = entity.into();
        let mut connection = self.pool.get()?;
        connection.transaction(|connection| {
            // Upserts keep the existing row identity and link its payload to that ID.
            data.id = diesel::insert_into(offer_details_structuralized::table)
                .values(&record)
                .on_conflict(offer_details_structuralized::offer_id)
                .do_update()
                .set(offer_details_structuralized::structuralized_at.eq(diesel::dsl::now))
                .returning(offer_details_structuralized::id)
                .get_result::<Uuid>(connection)?;
            diesel::insert_into(structuralized_offers_data::table)
                .values(&data)
                .on_conflict(structuralized_offers_data::id)
                .do_update()
                .set(structuralized_offers_data::document.eq(data.document))
                .execute(connection)?;
            Ok(())
        })
    }
}
