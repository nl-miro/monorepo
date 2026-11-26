//! Event listener for the Shopify GraphQL initial-fill saga.

use super::saga::ShopifyGraphQlCommand;
use crate::platform::bus::{Command, EventCommand, EventContext, event_command_envelope};
use crate::saga::model::InvalidSagaId;
use crate::saga::prelude::{DieselPgSagaRepository, ShopifyGraphQlInitialFillSaga};
use crate::saga::repository::{RepositoryError, SagaRepository};
use crate::steps::prelude::{
    ExtractOfferDetails, InitialFillRequested, OfferDetailsExtracted, OfferDetailsFetched,
    ProductCatalogIdsRetrieved, StorefrontTokenAcquired, StructuralizeOfferDetails,
};
use kernel::prelude::NewCommandEnvelope;
use retailer_sauce::prelude::StorefrontStrategy;
use serde_json::Error as JsonError;
use std::io::Error as IoError;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

pub(crate) struct Listener {
    // TODO:
    //     1. Make the listener generic — recommended here
    //
    //     pub(crate) struct Listener<R> {
    //     repository: R,
    //     }
    //
    //     impl<R> Listen for Listener<R>
    //     where
    //     R: SagaRepository + Send + Sync + 'static,
    //     {
    //     // Existing associated types and methods.
    // }
    //
    // Production uses Listener<DieselPgSagaRepository>; tests can use a fake repository. Your existing SagaRepository stays unchanged. The Send + Sync + 'static bounds are required by Listen.
    //
    // This matches the pattern already used by the export-message listener.
    // TODO:
    repository: Arc<DieselPgSagaRepository>,
}

#[derive(Debug, Error)]
pub(crate) enum ListenerError {
    #[error("initial fill event is missing its correlation ID")]
    MissingCorrelation,
    #[error("invalid initial fill correlation ID: {0}")]
    InvalidCorrelation(#[from] InvalidSagaId),
    #[error("could not prepare initial fill: {0}")]
    Preparation(#[from] IoError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("initial fill request conflicts with the stored saga {0:?}")]
    Conflict(Uuid),
}

#[rustfmt::skip]
impl Listener {
    pub(crate) fn new(repository: Arc<DieselPgSagaRepository>) -> Self {
        Self { repository }
    }

    pub(crate) fn on_initial_fill_requested(&self, event: InitialFillRequested, context: &EventContext) -> Result<(Uuid, Vec<ShopifyGraphQlCommand>), ListenerError> {

        let saga_id = event.saga_id();
        if event.strategy() != StorefrontStrategy::ShopifyGraphQl {
            return Ok((saga_id.into(), Vec::new()));
        }

        let correlation_id = context
            .correlation_id
            .ok_or(ListenerError::MissingCorrelation)?;

        let saga =
            ShopifyGraphQlInitialFillSaga::new(saga_id, correlation_id, event.retailer_code());

        let commands = saga.on_initial_fill_requested()?;

        match self.repository.insert(saga.clone()) {
            Ok(()) => {}
            Err(error) if error.is_duplicate() => {
                if self
                    .repository
                    .load::<ShopifyGraphQlInitialFillSaga>(saga_id)?
                    != saga
                {
                    return Err(ListenerError::Conflict(saga_id));
                }
            }
            Err(error) => return Err(error.into()),
        }

        Ok((saga_id.into(), commands))
    }

    pub(crate) fn on_storefront_token_acquired(&self, event: StorefrontTokenAcquired, context: &EventContext) -> Result<(Uuid, Vec<ShopifyGraphQlCommand>), ListenerError> {
        let saga = self.load_saga(context)?;
        let commands = saga.on_storefront_token_acquired(event.storefront_token)?;

        Ok((saga.saga_id().into(), commands))
    }

    pub(crate) fn on_product_catalog_ids_retrieved(&self, event: ProductCatalogIdsRetrieved, context: &EventContext) -> Result<(Uuid, Vec<ShopifyGraphQlCommand>), ListenerError> {

        let saga = self.load_saga(context)?;
        let commands = saga.on_product_catalog_ids_retrieved(
            event.product_ids,
            event.context,
            event.storefront_token,
        )?;

        Ok((saga.saga_id().into(), commands))
    }

    pub(crate) fn on_offer_details_fetched(&self, event: OfferDetailsFetched, context: &EventContext) -> Result<(Uuid, Vec<ShopifyGraphQlCommand>), ListenerError> {

        let saga = self.load_saga(context)?;
        let command = saga.on_offer_details_fetched(
            event.retailer_code,
            event.source,
            event.storage,
            event.fetched_at,
        );
        Ok((saga.saga_id().into(), vec![command]))
    }

    pub(crate) fn on_offer_details_extracted(&self, event: OfferDetailsExtracted, context: &EventContext) -> Result<(Uuid, Vec<ShopifyGraphQlCommand>), ListenerError> {

        let saga = self.load_saga(context)?;
        let command = saga.on_offer_details_extracted(
            event.retailer_code,
            event.source,
            event.storage,
            event.fetched_at,
        );
        Ok((saga.saga_id().into(), vec![command]))
    }

    fn load_saga(
        &self,
        context: &EventContext,
    ) -> Result<ShopifyGraphQlInitialFillSaga, ListenerError> {
        let correlation_id = context
            .correlation_id
            .ok_or(ListenerError::MissingCorrelation)?;

        Ok(self.repository.load(correlation_id)?)
    }
}

impl EventCommand for ShopifyGraphQlCommand {
    fn into_envelope(
        self,
        correlation_id: Uuid,
        event_name: &str,
        context: &EventContext,
    ) -> Result<NewCommandEnvelope, JsonError> {
        match self {
            Self::Token(command) => event_command_envelope(
                &command,
                Uuid::now_v7(),
                correlation_id,
                event_name,
                context,
            ),
            Self::CatalogProductIds(command) => event_command_envelope(
                &command,
                Uuid::now_v7(),
                correlation_id,
                event_name,
                context,
            ),
            Self::FetchOfferDetails(command) => event_command_envelope(
                &command,
                Uuid::now_v7(),
                correlation_id,
                event_name,
                context,
            ),
            Self::ExtractOfferDetails(command) => {
                let id = Uuid::new_v5(
                    &command.storage.object_storage_id,
                    ExtractOfferDetails::NAME.as_bytes(),
                );
                event_command_envelope(&command, id, correlation_id, event_name, context)
            }
            Self::StructuralizeOfferDetails(command) => {
                let id = Uuid::new_v5(
                    &command.storage.object_storage_id,
                    StructuralizeOfferDetails::NAME.as_bytes(),
                );
                event_command_envelope(&command, id, correlation_id, event_name, context)
            }
        }
    }
}
