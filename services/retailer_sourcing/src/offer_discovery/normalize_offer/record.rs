//! Private PostgreSQL representations and domain mappings.
use super::super::schema::{normalized_offers_data, offer_details_normalized};
use super::entity::{ExtractedOffer, NormalizedOffer};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

/// Projection of the input row joined to its payload.
#[derive(Queryable)]
pub(super) struct ExtractedOfferPgRecord {
    pub offer_id: Uuid,
    pub document: Value,
}

impl From<ExtractedOfferPgRecord> for ExtractedOffer {
    fn from(record: ExtractedOfferPgRecord) -> Self {
        Self {
            offer_id: record.offer_id,
            document: record.document,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = offer_details_normalized)]
pub(super) struct NormalizedOfferPgRecord {
    pub id: Uuid,
    pub offer_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = normalized_offers_data)]
pub(super) struct NormalizedOfferDataPgRecord<'a> {
    pub id: Uuid,
    pub document: &'a Value,
}

impl<'a> From<&'a NormalizedOffer> for (NormalizedOfferPgRecord, NormalizedOfferDataPgRecord<'a>) {
    fn from(entity: &'a NormalizedOffer) -> Self {
        let id = Uuid::now_v7();
        (
            NormalizedOfferPgRecord {
                id,
                offer_id: entity.offer_id,
            },
            NormalizedOfferDataPgRecord {
                id,
                document: &entity.document,
            },
        )
    }
}
