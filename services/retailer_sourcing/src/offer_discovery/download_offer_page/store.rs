use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use super::super::schema::{offer_details_raw, raw_offers_data};
use super::entity::RawOffer;
use super::handler::Store;
use super::record::{RawOfferDataPgRecord, RawOfferPgRecord};
use crate::platform::db::{self, DbPool};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),

    #[error("offer page body is too large for storage: {0} bytes")]
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
    fn store_raw_offer(&self, entity: &RawOffer) -> Result<(), StoreError> {
        let (record, mut data): (RawOfferPgRecord<'_>, RawOfferDataPgRecord<'_>) =
            entity.try_into()?;
        let mut connection = self.pool.get()?;
        connection.transaction(|connection| {
            data.id = diesel::insert_into(offer_details_raw::table)
                .values(&record)
                .on_conflict(offer_details_raw::offer_id)
                .do_update()
                .set((
                    offer_details_raw::retailer_code.eq(&record.retailer_code),
                    offer_details_raw::body_size.eq(record.body_size),
                    offer_details_raw::fetched_at.eq(diesel::dsl::now),
                ))
                .returning(offer_details_raw::id)
                .get_result::<Uuid>(connection)?;
            diesel::insert_into(raw_offers_data::table)
                .values(&data)
                .on_conflict(raw_offers_data::id)
                .do_update()
                .set(raw_offers_data::body.eq(data.body))
                .execute(connection)?;
            Ok(())
        })
    }
}
