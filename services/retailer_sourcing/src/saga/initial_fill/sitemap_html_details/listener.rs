//! Event listener for the sitemap and HTML details initial-fill saga.

use super::saga::StartingCommand;
use crate::platform::bus::{EventCommand, EventContext, event_command_envelope};
use crate::saga::prelude::{
    DieselPgSagaRepository, RepositoryError, SagaRepository,
    SitemapHtmlDetailsInitialFillSaga,
};
use crate::steps::prelude::InitialFillRequested;
use crate::steps::prelude::{
    OfferDiscovered, OfferPageWasDownloaded, OfferPageWasExtracted, OfferWasNormalized,
    SitemapConsolidated, SitemapProcessed, SitemapRetrieved,
};
use diesel::result::Error as DieselError;
use kernel::prelude::NewCommandEnvelope;
use retailer_sauce::prelude::StorefrontStrategy;
use serde_json::Error as JsonError;
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

impl Listener {
    pub(crate) fn new(repository: Arc<DieselPgSagaRepository>) -> Self {
        Self { repository }
    }

    pub(crate) fn on_initial_fill_requested(
        &self,
        event: InitialFillRequested,
        context: &EventContext,
    ) -> Result<(Uuid, Vec<StartingCommand>), ListenerError> {
        let saga_id = event.saga_id();

        if event.strategy() != StorefrontStrategy::ShopifySitemapPlusHtml {
            return Ok((saga_id.into(), vec![]));
        }

        let correlation_id = context
            .correlation_id
            .ok_or(ListenerError::MissingCorrelation)?;

        let saga =
            SitemapHtmlDetailsInitialFillSaga::new(saga_id, correlation_id, event.retailer_code());

        let next_step = saga.on_initial_fill_requested();

        match self.repository.insert(saga.clone()) {
            Ok(()) => {}
            Err(error) if error.is_duplicate() => {
                if self
                    .repository
                    .load::<SitemapHtmlDetailsInitialFillSaga>(saga_id)?
                    != saga
                {
                    return Err(ListenerError::Conflict(saga_id));
                }
            }
            Err(error) => return Err(error.into()),
        }

        Ok((saga_id.into(), vec![next_step]))
    }

    pub(crate) fn on_sitemap_retrieved(
        &self,
        event: SitemapRetrieved,
        context: &EventContext,
    ) -> Result<(Uuid, Vec<StartingCommand>), ListenerError> {
        let saga = self.load_saga(context)?;
        let command = saga.on_sitemap_retrieved(event);
        Ok((saga.saga_id().into(), vec![command]))
    }

    pub(crate) fn on_sitemap_processed(
        &self,
        event: SitemapProcessed,
        context: &EventContext,
    ) -> Result<(Uuid, Vec<StartingCommand>), ListenerError> {
        let saga = self.load_saga(context)?;
        let command = saga.on_sitemap_processed(event);
        Ok((saga.saga_id().into(), vec![command]))
    }

    pub(crate) fn on_sitemap_consolidated(
        &self,
        event: SitemapConsolidated,
        context: &EventContext,
    ) -> Result<(Uuid, Vec<StartingCommand>), ListenerError> {
        let saga = self.load_saga(context)?;
        let commands = saga.on_sitemap_consolidated(event);
        Ok((saga.saga_id().into(), commands))
    }

    pub(crate) fn on_offer_discovered(
        &self,
        event: OfferDiscovered,
        context: &EventContext,
    ) -> Result<(Uuid, Vec<StartingCommand>), ListenerError> {
        let saga = self.load_saga(context)?;
        let command = saga.on_offer_discovered(event);
        Ok((saga.saga_id().into(), vec![command]))
    }

    pub(crate) fn on_offer_page_was_downloaded(
        &self,
        event: OfferPageWasDownloaded,
        context: &EventContext,
    ) -> Result<(Uuid, Vec<StartingCommand>), ListenerError> {
        let saga = self.load_saga(context)?;
        let command = saga.on_offer_page_was_downloaded(event);
        Ok((saga.saga_id().into(), vec![command]))
    }

    pub(crate) fn on_offer_page_was_extracted(
        &self,
        event: OfferPageWasExtracted,
        context: &EventContext,
    ) -> Result<(Uuid, Vec<StartingCommand>), ListenerError> {
        let saga = self.load_saga(context)?;
        let command = saga.on_offer_page_was_extracted(event);
        Ok((saga.saga_id().into(), vec![command]))
    }

    pub(crate) fn on_offer_was_normalized(
        &self,
        event: OfferWasNormalized,
        context: &EventContext,
    ) -> Result<(Uuid, Vec<StartingCommand>), ListenerError> {
        let saga = self.load_saga(context)?;
        let command = saga.on_offer_was_normalized(event);
        Ok((saga.saga_id().into(), vec![command]))
    }

    fn load_saga(
        &self,
        context: &EventContext,
    ) -> Result<SitemapHtmlDetailsInitialFillSaga, ListenerError> {
        let correlation_id = context
            .correlation_id
            .ok_or(ListenerError::MissingCorrelation)?;
        Ok(self.repository.load(correlation_id)?)
    }
}

#[derive(Debug, Error)]
pub(crate) enum ListenerError {
    #[error("initial fill event is missing its correlation ID")]
    MissingCorrelation,
    #[error("initial fill correlation ID must be a UUIDv7")]
    InvalidCorrelation,
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("initial fill request conflicts with the stored saga {0:?}")]
    Conflict(Uuid),
}

impl ListenerError {
    pub(crate) fn is_standalone(&self) -> bool {
        matches!(
            self,
            Self::MissingCorrelation
                | Self::InvalidCorrelation
                | Self::Repository(RepositoryError::Database(DieselError::NotFound))
        )
    }
}

impl EventCommand for StartingCommand {
    fn into_envelope(
        self,
        correlation_id: Uuid,
        event_name: &str,
        context: &EventContext,
    ) -> Result<NewCommandEnvelope, JsonError> {
        match self {
            // Each sitemap event yields at most one command. Reuse its ID on redelivery.
            Self::RetrieveSitemap(command) => event_command_envelope(
                &command,
                context.event_id,
                correlation_id,
                event_name,
                context,
            ),
            Self::ProcessSitemap(command) => event_command_envelope(
                &command,
                context.event_id,
                correlation_id,
                event_name,
                context,
            ),
            Self::ConsolidateSitemap(command) => event_command_envelope(
                &command,
                context.event_id,
                correlation_id,
                event_name,
                context,
            ),
            Self::DiscoverOffers(command) => event_command_envelope(
                &command,
                context.event_id,
                correlation_id,
                event_name,
                context,
            ),
            Self::DownloadOfferPage(command) => event_command_envelope(
                &command,
                context.event_id,
                correlation_id,
                event_name,
                context,
            ),
            Self::ExtractOfferPage(command) => event_command_envelope(
                &command,
                context.event_id,
                correlation_id,
                event_name,
                context,
            ),
            Self::NormalizeOffer(command) => event_command_envelope(
                &command,
                context.event_id,
                correlation_id,
                event_name,
                context,
            ),
            Self::StructuralizeOffer(command) => event_command_envelope(
                &command,
                context.event_id,
                correlation_id,
                event_name,
                context,
            ),
        }
    }
}
