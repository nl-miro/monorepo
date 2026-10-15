//! Private PostgreSQL representations and domain mappings.
use super::super::schema::{extracted_offers_data, offer_details_extracted};
use super::entity::{ExtractedOffer, RawOffer};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

/// Projection of the input row joined to its payload.
#[derive(Queryable)]
pub(super) struct RawOfferPgRecord {
    pub offer_id: Uuid,
    pub body: String,
}

impl From<RawOfferPgRecord> for RawOffer {
    fn from(record: RawOfferPgRecord) -> Self {
        Self {
            offer_id: record.offer_id,
            body: record.body,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = offer_details_extracted)]
pub(super) struct ExtractedOfferPgRecord {
    pub id: Uuid,
    pub offer_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = extracted_offers_data)]
pub(super) struct ExtractedOfferDataPgRecord<'a> {
    pub id: Uuid,
    pub document: &'a Value,
}

impl<'a> From<&'a ExtractedOffer> for (ExtractedOfferPgRecord, ExtractedOfferDataPgRecord<'a>) {
    fn from(entity: &'a ExtractedOffer) -> Self {
        let id = Uuid::now_v7();
        (
            ExtractedOfferPgRecord {
                id,
                offer_id: entity.offer_id,
            },
            ExtractedOfferDataPgRecord {
                id,
                document: &entity.document,
            },
        )
    }
}
