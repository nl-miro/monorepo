use super::{
    RetailerOfferIdentifiers, RetailerOfferIdentifiersRepository,
    RetailerOfferIdentifiersRepositoryError, err_identity_missing, err_insert, err_update,
    validate, validate_update,
};
use crate::offers::{
    handler::{
        RetailerOfferIdentifiersError,
        RetailerOfferIdentifiersRepository as IdentityLookupRepository,
    },
    retailer_offer_id::RetailerOfferId,
};
use bodul_shared::retailer::RetailerCode;
use diesel::{
    OptionalExtension, RunQueryDsl,
    sql_types::{Nullable, Text, Uuid as SqlUuid},
};
use retailer_sauce::identifier::{
    Gtin, OfferIdentifiers, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku,
};
use testimony_infra::prelude::DbPool;
use uuid::Uuid;

pub struct PostgresRetailerOfferIdentifiersRepository {
    pool: DbPool,
}

impl PostgresRetailerOfferIdentifiersRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl RetailerOfferIdentifiersRepository for PostgresRetailerOfferIdentifiersRepository {
    fn insert(
        &self,
        entity: &RetailerOfferIdentifiers,
    ) -> Result<(), RetailerOfferIdentifiersRepositoryError> {
        let id = validate(entity)?;
        let mut connection = self.pool.get().map_err(err_insert)?;
        diesel::sql_query(
            "INSERT INTO read_offer_identities (id, retailer_code, url, gtin, sku, shopify_product_gid, shopify_product_variation_gid, shopify_handle) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(entity.retailer_code.as_str())
        .bind::<Text, _>(&entity.url)
        .bind::<Nullable<Text>, _>(entity.identifiers.gtin.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(entity.identifiers.sku.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(entity.identifiers.shopify_product_gid.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(entity.identifiers.shopify_product_variation_gid.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(entity.identifiers.shopify_handle.as_ref().map(|value| &value.value))
        .execute(&mut connection)
        .map_err(err_insert)?;
        Ok(())
    }

    fn update_identifiers(
        &self,
        id: &crate::offers::retailer_offer_id::RetailerOfferId,
        identifiers: &retailer_sauce::identifier::OfferIdentifiers,
    ) -> Result<(), RetailerOfferIdentifiersRepositoryError> {
        let id = validate_update(id, identifiers)?;
        let count = diesel::sql_query(
            "UPDATE read_offer_identities SET gtin=$2, sku=$3, shopify_product_gid=$4, shopify_product_variation_gid=$5, shopify_handle=$6 WHERE id=$1",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Nullable<Text>, _>(identifiers.gtin.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(identifiers.sku.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(identifiers.shopify_product_gid.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(identifiers.shopify_product_variation_gid.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(identifiers.shopify_handle.as_ref().map(|value| &value.value))
        .execute(&mut self.pool.get().map_err(err_update)?)
        .map_err(err_update)?;
        if count == 0 {
            return Err(err_identity_missing());
        }
        Ok(())
    }
}

impl IdentityLookupRepository for PostgresRetailerOfferIdentifiersRepository {
    fn insert(
        &self,
        id: &RetailerOfferId,
        retailer_code: RetailerCode,
        url: &str,
    ) -> Result<(), RetailerOfferIdentifiersError> {
        let entity = RetailerOfferIdentifiers {
            id: id.clone(),
            retailer_code,
            url: url.to_owned(),
            identifiers: OfferIdentifiers::default(),
        };
        RetailerOfferIdentifiersRepository::insert(self, &entity).map_err(err_identity_repository)
    }

    fn find(
        &self,
        retailer_code: RetailerCode,
        url: &str,
    ) -> Result<Option<RetailerOfferIdentifiers>, RetailerOfferIdentifiersError> {
        #[derive(diesel::QueryableByName)]
        struct IdentityRow {
            #[diesel(sql_type = SqlUuid)]
            id: Uuid,
            #[diesel(sql_type = Nullable<Text>)]
            gtin: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            sku: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            shopify_product_gid: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            shopify_product_variation_gid: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            shopify_handle: Option<String>,
        }

        let mut connection = self.pool.get().map_err(err_identity_repository)?;
        let row = diesel::sql_query(
            "SELECT id, gtin, sku, shopify_product_gid, shopify_product_variation_gid, shopify_handle FROM read_offer_identities WHERE retailer_code = $1 AND url = $2",
        )
        .bind::<Text, _>(retailer_code.as_str())
        .bind::<Text, _>(url)
        .get_result::<IdentityRow>(&mut connection)
        .optional()
        .map_err(err_identity_repository)?;

        row.map(|row| {
            let identifiers = OfferIdentifiers {
                gtin: row
                    .gtin
                    .map(Gtin::try_from)
                    .transpose()
                    .map_err(err_identity_repository)?,
                sku: row
                    .sku
                    .map(Sku::try_from)
                    .transpose()
                    .map_err(err_identity_repository)?,
                shopify_product_gid: row
                    .shopify_product_gid
                    .map(ShopifyProductGid::try_from)
                    .transpose()
                    .map_err(err_identity_repository)?,
                shopify_product_variation_gid: row
                    .shopify_product_variation_gid
                    .map(ShopifyProductVariationGid::try_from)
                    .transpose()
                    .map_err(err_identity_repository)?,
                shopify_handle: row
                    .shopify_handle
                    .map(ShopifyHandle::try_from)
                    .transpose()
                    .map_err(err_identity_repository)?,
            };
            identifiers.guard().map_err(err_identity_repository)?;
            let identity = RetailerOfferIdentifiers {
                identifiers,
                id: RetailerOfferId::new(row.id.to_string()),
                retailer_code,
                url: url.to_owned(),
            };
            Ok(identity)
        })
        .transpose()
    }
}

fn err_identity_repository(error: impl std::fmt::Display) -> RetailerOfferIdentifiersError {
    RetailerOfferIdentifiersError(error.to_string())
}
