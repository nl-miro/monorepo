//! Saga domain entities.

use super::model::SagaId;
use crate::saga::prelude::RecordableSaga;
use crate::saga::record::{DieselPgSagaRecord, SagaRecordError, SagaTypeMismatch};
use crate::steps::prelude::{
    AcquireStorefrontToken, ExtractOfferDetails, FetchOfferDetails, FetchProductCatalogIdentifiers,
    OfferDetailsExtracted, OfferDetailsFetched, ProductCatalogIdsRetrieved, SourceType,
    StorefrontTokenAcquired, StructuralizeOfferDetails,
};
use retailer_data_extractor::load_storefront_config;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::{StorefrontConfig, StorefrontToken};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::retailer::RetailerCode;
use std::io::Error as IoError;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShopifyGraphQlInitialFillSaga {
    saga_id: SagaId,
    correlation_id: Uuid,
    retailer_code: RetailerCode,
}

impl RecordableSaga for ShopifyGraphQlInitialFillSaga {}

impl ShopifyGraphQlInitialFillSaga {
    pub(crate) const SAGA_TYPE: &'static str = "ShopifyGraphQlInitialFill";

    pub(super) fn saga_id(&self) -> SagaId {
        self.saga_id
    }

    pub(crate) fn new(saga_id: SagaId, correlation_id: Uuid, retailer_code: RetailerCode) -> Self {
        Self {
            saga_id,
            correlation_id,
            retailer_code,
        }
    }
}

pub(crate) enum StartingCommand {
    Token(AcquireStorefrontToken),
    CatalogProductIds(FetchProductCatalogIdentifiers),
    FetchOfferDetails(FetchOfferDetails),
    ExtractOfferDetails(ExtractOfferDetails),
    StructuralizeOfferDetails(StructuralizeOfferDetails),
}

impl ShopifyGraphQlInitialFillSaga {
    pub(crate) fn on_initial_fill_requested(&self) -> Result<Vec<StartingCommand>, std::io::Error> {
        let cfg = load_storefront_config(self.retailer_code)?;

        let res = if cfg.attempts_token() {
            let cmd = cfg.try_into()?;

            vec![StartingCommand::Token(cmd)]
        } else {
            let commands = self.build_fetch_identifiers(cfg, None)?;
            commands
                .into_iter()
                .map(StartingCommand::CatalogProductIds)
                .collect()
        };

        Ok(res)
    }

    #[rustfmt::skip]
    fn build_fetch_identifiers(&self,cfg: StorefrontConfig,storefront_token: Option<StorefrontToken>) -> Result<Vec<FetchProductCatalogIdentifiers>, IoError> {

        let contexts = cfg.initial_fill_contexts().map_err(IoError::other)?;

        let commands = contexts
            .into_iter()
            .map(|context| self.build_fetch_identifier(context, storefront_token.clone()))
            .collect();

        Ok(commands)
    }

    #[rustfmt::skip]
    fn build_fetch_identifier(&self,context: RequestedContext,storefront_token: Option<StorefrontToken>) -> FetchProductCatalogIdentifiers {

        FetchProductCatalogIdentifiers {
            context: Some(context),
            retailer_code: self.retailer_code,
            storefront_token,
        }
    }
}

impl TryFrom<StorefrontConfig> for AcquireStorefrontToken {
    type Error = std::io::Error;

    fn try_from(cfg: StorefrontConfig) -> Result<Self, Self::Error> {
        let cmd = AcquireStorefrontToken::new(cfg.retailer_code(), cfg.storefront_url().into());

        Ok(cmd)
    }
}

impl ShopifyGraphQlInitialFillSaga {
    #[rustfmt::skip]
    pub(super) fn on_storefront_token_acquired(&self, event: StorefrontTokenAcquired) -> Result<Vec<StartingCommand>, std::io::Error> {

        let cfg = load_storefront_config(self.retailer_code)?;

        let commands = self.build_fetch_identifiers(cfg, event.storefront_token)?;

        let res = commands
            .into_iter()
            .map(StartingCommand::CatalogProductIds)
            .collect();

        Ok(res)
    }

    pub(super) fn on_product_catalog_ids_retrieved(
        &self,
        event: ProductCatalogIdsRetrieved,
    ) -> Result<Vec<StartingCommand>, std::io::Error> {
        let mut commands = vec![];

        for product_id in event.product_ids {
            let cmd = FetchOfferDetails {
                context: event.context.clone(),
                storefront_token: event.storefront_token.clone(),
                retailer_code: event.retailer_code,
                source: SourceType::GraphQl { product_id },
            };

            commands.push(cmd);
        }

        let res = commands
            .into_iter()
            .map(StartingCommand::FetchOfferDetails)
            .collect();

        Ok(res)
    }

    pub(super) fn on_offer_details_fetched(&self, event: OfferDetailsFetched) -> StartingCommand {
        let cmd = ExtractOfferDetails {
            retailer_code: event.retailer_code,
            source: event.source,
            storage: event.storage,
            fetched_at: event.fetched_at,
        };

        StartingCommand::ExtractOfferDetails(cmd)
    }

    pub(super) fn on_offer_details_extracted(
        &self,
        event: OfferDetailsExtracted,
    ) -> StartingCommand {
        let cmd = StructuralizeOfferDetails {
            retailer_code: event.retailer_code,
            source: event.source,
            storage: event.storage,
            fetched_at: event.fetched_at,
        };

        StartingCommand::StructuralizeOfferDetails(cmd)
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
