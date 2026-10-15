use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use super::super::schema::{
    extracted_offers_data, offer_details_extracted, offer_details_raw, raw_offers_data,
};
use super::entity::{ExtractedOffer, RawOffer};
use super::handler::Store;
use super::record::{ExtractedOfferDataPgRecord, ExtractedOfferPgRecord, RawOfferPgRecord};
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
    fn raw_offer(&self, offer_id: Uuid) -> Result<Option<RawOffer>, StoreError> {
        let mut connection = self.pool.get()?;
        let record = offer_details_raw::table
            .inner_join(raw_offers_data::table)
            .filter(offer_details_raw::offer_id.eq(offer_id))
            .select((offer_details_raw::offer_id, raw_offers_data::body))
            .first::<RawOfferPgRecord>(&mut connection)
            .optional()?;
        Ok(record.map(Into::into))
    }

    fn store_extracted_offer(&self, entity: &ExtractedOffer) -> Result<(), StoreError> {
        let (record, mut data): (ExtractedOfferPgRecord, ExtractedOfferDataPgRecord<'_>) =
            entity.into();
        let mut connection = self.pool.get()?;
        connection.transaction(|connection| {
            // Upserts keep the existing row identity and link its payload to that ID.
            data.id = diesel::insert_into(offer_details_extracted::table)
                .values(&record)
                .on_conflict(offer_details_extracted::offer_id)
                .do_update()
                .set(offer_details_extracted::extracted_at.eq(diesel::dsl::now))
                .returning(offer_details_extracted::id)
                .get_result::<Uuid>(connection)?;
            diesel::insert_into(extracted_offers_data::table)
                .values(&data)
                .on_conflict(extracted_offers_data::id)
                .do_update()
                .set(extracted_offers_data::document.eq(data.document))
                .execute(connection)?;
            Ok(())
        })
    }
}
