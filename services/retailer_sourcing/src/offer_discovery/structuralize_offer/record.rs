//! Private PostgreSQL representations and domain mappings.
use super::super::schema::{offer_details_structuralized, structuralized_offers_data};
use super::entity::{NormalizedOffer, StructuralizedOffer};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

/// Projection of the input row joined to its payload.
#[derive(Queryable)]
pub(super) struct NormalizedOfferPgRecord {
    pub offer_id: Uuid,
    pub document: Value,
}

impl From<NormalizedOfferPgRecord> for NormalizedOffer {
    fn from(record: NormalizedOfferPgRecord) -> Self {
        Self {
            offer_id: record.offer_id,
            document: record.document,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = offer_details_structuralized)]
pub(super) struct StructuralizedOfferPgRecord {
    pub id: Uuid,
    pub offer_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = structuralized_offers_data)]
pub(super) struct StructuralizedOfferDataPgRecord<'a> {
    pub id: Uuid,
    pub document: &'a Value,
}

impl<'a> From<&'a StructuralizedOffer>
    for (
        StructuralizedOfferPgRecord,
        StructuralizedOfferDataPgRecord<'a>,
    )
{
    fn from(entity: &'a StructuralizedOffer) -> Self {
        let id = Uuid::now_v7();
        (
            StructuralizedOfferPgRecord {
                id,
                offer_id: entity.offer_id,
            },
            StructuralizedOfferDataPgRecord {
                id,
                document: &entity.document,
            },
        )
    }
}
