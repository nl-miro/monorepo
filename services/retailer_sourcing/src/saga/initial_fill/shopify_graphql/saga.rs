//! Saga domain entities.

use crate::platform::object_storage::OfferStorageMetadata;
use crate::saga::initial_fill::shopify_graphql::builders::{
    build_acquire_storefront_token, build_extract_offer_details, build_fetch_identifiers,
    build_fetch_offer_details, build_structuralize_offer_details,
};
use crate::saga::prelude::RecordableSaga;
use crate::saga::record::{DieselPgSagaRecord, SagaRecordError, SagaTypeMismatch};
use crate::steps::prelude::{
    AcquireStorefrontToken, ExtractOfferDetails, FetchOfferDetails, FetchProductCatalogIdentifiers,
    SourceType, StructuralizeOfferDetails,
};
use chrono::{DateTime, Utc};
use retailer_data_extractor::load_storefront_config;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::StorefrontToken;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShopifyGraphQlInitialFillSaga {
    saga_id: Uuid,
    correlation_id: Uuid,
    retailer_code: RetailerCode,
}

impl RecordableSaga for ShopifyGraphQlInitialFillSaga {}

impl ShopifyGraphQlInitialFillSaga {
    pub(crate) const SAGA_TYPE: &'static str = "ShopifyGraphQlInitialFill";

    pub(super) fn saga_id(&self) -> Uuid {
        self.saga_id
    }

    pub(crate) fn new(saga_id: Uuid, correlation_id: Uuid, retailer_code: RetailerCode) -> Self {
        Self {
            saga_id,
            correlation_id,
            retailer_code,
        }
    }
}

pub(crate) enum ShopifyGraphQlCommand {
    Token(AcquireStorefrontToken),
    CatalogProductIds(FetchProductCatalogIdentifiers),
    FetchOfferDetails(FetchOfferDetails),
    ExtractOfferDetails(ExtractOfferDetails),
    StructuralizeOfferDetails(StructuralizeOfferDetails),
}

#[rustfmt::skip]
impl ShopifyGraphQlInitialFillSaga {
    pub(crate) fn on_initial_fill_requested(&self) -> Result<Vec<ShopifyGraphQlCommand>, std::io::Error> {
        let cfg = load_storefront_config(self.retailer_code)?;

        if cfg.attempts_token() {
            let cmd = build_acquire_storefront_token(cfg);

            Ok(vec![cmd])
        } else {
            build_fetch_identifiers(cfg, None)
        }
    }

    pub(super) fn on_storefront_token_acquired(&self, storefront_token: Option<StorefrontToken>) -> Result<Vec<ShopifyGraphQlCommand>, std::io::Error> {

        let cfg = load_storefront_config(self.retailer_code)?;

        build_fetch_identifiers(cfg, storefront_token)
    }

    pub(super) fn on_product_catalog_ids_retrieved(&self, product_ids: Vec<String>, context: Option<RequestedContext>, token: Option<StorefrontToken>) -> Result<Vec<ShopifyGraphQlCommand>, std::io::Error> {

        build_fetch_offer_details(self.retailer_code, product_ids, context, token)
    }

    pub(super) fn on_offer_details_fetched(&self, retailer_code: RetailerCode, source: SourceType, storage: OfferStorageMetadata, acquired_at: DateTime<Utc>) -> ShopifyGraphQlCommand {

        build_extract_offer_details(retailer_code, source, storage, acquired_at)
    }

    pub(super) fn on_offer_details_extracted(&self, retailer_code: RetailerCode, source: SourceType, storage: OfferStorageMetadata, acquired_at: DateTime<Utc>) -> ShopifyGraphQlCommand {

        build_structuralize_offer_details(retailer_code, source, storage, acquired_at)
    }
}

impl TryFrom<ShopifyGraphQlInitialFillSaga> for DieselPgSagaRecord {
    type Error = SagaTypeMismatch;

    fn try_from(fill: ShopifyGraphQlInitialFillSaga) -> Result<Self, Self::Error> {
        let res = DieselPgSagaRecord {
            saga_id: fill.saga_id,
            correlation_id: fill.correlation_id,
            saga_type: ShopifyGraphQlInitialFillSaga::SAGA_TYPE.into(),
            payload: json!({"retailer_code": fill.retailer_code}),
        };

        Ok(res)
    }
}

impl TryFrom<DieselPgSagaRecord> for ShopifyGraphQlInitialFillSaga {
    type Error = SagaRecordError;

    fn try_from(record: DieselPgSagaRecord) -> Result<Self, Self::Error> {
        if record.saga_type != Self::SAGA_TYPE {
            let err = SagaTypeMismatch::new(Self::SAGA_TYPE, record.saga_type);
            return Err(err.into());
        }

        let retailer_code = RetailerCode::deserialize(&record.payload["retailer_code"])?;
        let res = Self::new(record.saga_id, record.correlation_id, retailer_code);

        Ok(res)
    }
}
