//! PostgreSQL mappings for downloaded offer pages.
use super::super::schema::{offer_details_raw, raw_offers_data};
use super::entity::RawOffer;
use super::store::StoreError;
use diesel::Insertable;
use uuid::Uuid;

#[derive(Insertable)]
#[diesel(table_name = offer_details_raw)]
pub(super) struct RawOfferPgRecord<'a> {
    pub id: Uuid,
    pub offer_id: Uuid,
    pub url: &'a str,
    pub body_size: i32,
    pub retailer_code: String,
}

#[derive(Insertable)]
#[diesel(table_name = raw_offers_data)]
pub(super) struct RawOfferDataPgRecord<'a> {
    pub id: Uuid,
    pub body: &'a str,
}

impl<'a> TryFrom<&'a RawOffer> for (RawOfferPgRecord<'a>, RawOfferDataPgRecord<'a>) {
    type Error = StoreError;
    fn try_from(entity: &'a RawOffer) -> Result<Self, Self::Error> {
        let id = Uuid::now_v7();
        Ok((
            RawOfferPgRecord {
                id,
                offer_id: entity.offer_id,
                url: &entity.url,
                body_size: stored_size(entity.body.len())?,
                retailer_code: entity.retailer_code.as_str(),
            },
            RawOfferDataPgRecord {
                id,
                body: &entity.body,
            },
        ))
    }
}

fn stored_size(size: usize) -> Result<i32, StoreError> {
    i32::try_from(size).map_err(|_| StoreError::BodyTooLarge(size))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_storage_size_bounds() {
        assert_eq!(stored_size(0).unwrap(), 0);
        assert_eq!(stored_size(i32::MAX as usize).unwrap(), i32::MAX);
        assert!(matches!(
            stored_size(i32::MAX as usize + 1),
            Err(StoreError::BodyTooLarge(_))
        ));
    }
}
