//! Initialization routes by the original event-envelope correlation.

use super::results::load_result;
use super::{ApplicationCommand, ShopifyGraphQlInitialFillSaga};
use crate::platform::bus::EventEnvelope;
use crate::saga::saga::SagaRecord;
use crate::saga::saga_repository::{SagaRepository, SaveError};
use crate::steps::graphql::acquire_storefront_token::StorefrontTokenAcquired;
use crate::steps::graphql::extract_offer_details::{OfferDetailsExtracted, extract};
use crate::steps::graphql::offer_details::OfferDetailsFetched;
use crate::steps::graphql::product_catalog_ids::ProductCatalogIdsRetrieved;
use crate::steps::graphql::structuralize_offer_details::{
    OfferDetailsStructuralized, structuralize,
};
use crate::steps::prelude::RetailerFillInitialized;
use diesel::PgConnection;
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Clone)]
pub(super) struct Listener {
    repository: SagaRepository,
}
impl Listener {
    pub(super) fn new(repository: SagaRepository) -> Self {
        Self { repository }
    }
    pub(super) fn on_retailer_fill_initialized(
        &self,
        envelope: &EventEnvelope<RetailerFillInitialized>,
    ) -> Result<Vec<ApplicationCommand>, String> {
        let correlation = envelope
            .metadata
            .as_ref()
            .and_then(|meta| meta.correlation_id)
            .ok_or("initialization requires envelope correlation")?;
        let event_id = envelope
            .metadata
            .as_ref()
            .expect("checked metadata")
            .event_id;
        let retailer = envelope.event.retailer_code;
        // Replay must not read live inventory. The transaction validates correlation
        // even when no record exists; concurrent creation converges on its winner.
        let existing = self
            .repository
            .transition::<ShopifyGraphQlInitialFillSaga, _>(
                correlation,
                retailer,
                None,
                |record, _| {
                    Ok(record
                        .payload
                        .initial_commands(record.id, record.retailer_code, event_id))
                },
            )
            .map_err(|error| error.to_string())?;
        if let Some(commands) = existing {
            return Ok(commands);
        }
        let initial = ShopifyGraphQlInitialFillSaga::snapshot(retailer)?;
        self.repository
            .transition(correlation, retailer, Some(initial), |record, _| {
                Ok(record
                    .payload
                    .initial_commands(record.id, record.retailer_code, event_id))
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "initialization did not create a saga".into())
    }
    fn apply<E>(
        &self,
        envelope: &EventEnvelope<E>,
        retailer: RetailerCode,
        apply: impl FnOnce(
            &mut SagaRecord<ShopifyGraphQlInitialFillSaga>,
            &mut PgConnection,
            Uuid,
            Uuid,
        ) -> Result<Vec<ApplicationCommand>, String>,
    ) -> Result<Vec<ApplicationCommand>, String> {
        let meta = envelope
            .metadata
            .as_ref()
            .ok_or("step event requires metadata")?;
        let correlation = meta
            .correlation_id
            .ok_or("step event requires correlation")?;
        let cause = meta.causation_id.ok_or("step event requires causation")?;
        self.repository
            .transition::<ShopifyGraphQlInitialFillSaga, _>(
                correlation,
                retailer,
                None,
                |record, conn| {
                    let commands =
                        apply(record, conn, cause, meta.event_id).map_err(SaveError::Invalid)?;
                    record.status = record.payload.status().into();
                    Ok(commands)
                },
            )
            .map(|result| result.unwrap_or_default())
            .map_err(|error| error.to_string())
    }
    pub(super) fn on_token(
        &self,
        envelope: &EventEnvelope<StorefrontTokenAcquired>,
    ) -> Result<Vec<ApplicationCommand>, String> {
        self.apply(
            envelope,
            envelope.event.retailer_code,
            |record, _, cause, event_id| {
                record.payload.on_token(
                    record.id,
                    record.retailer_code,
                    cause,
                    event_id,
                    &envelope.event,
                )
            },
        )
    }
    pub(super) fn on_catalog(
        &self,
        envelope: &EventEnvelope<ProductCatalogIdsRetrieved>,
    ) -> Result<Vec<ApplicationCommand>, String> {
        self.apply(
            envelope,
            envelope.event.retailer_code,
            |record, _, cause, event_id| {
                record.payload.on_catalog(
                    record.id,
                    record.retailer_code,
                    cause,
                    event_id,
                    &envelope.event,
                )
            },
        )
    }
    pub(super) fn on_fetched(
        &self,
        envelope: &EventEnvelope<OfferDetailsFetched>,
    ) -> Result<Vec<ApplicationCommand>, String> {
        self.apply(
            envelope,
            envelope.event.retailer_code,
            |record, conn, cause, event_id| {
                let Some((index, product)) =
                    record
                        .payload
                        .locate_fetch(record.id, record.retailer_code, cause)
                else {
                    return Ok(vec![]);
                };
                let Some(reference) =
                    record
                        .payload
                        .accept_fetch(index, &product, event_id, &envelope.event)?
                else {
                    return Ok(vec![]);
                };
                let canonical: OfferDetailsFetched = load_result(conn, reference.event)?;
                Ok(extract(&canonical)
                    .into_iter()
                    .map(ApplicationCommand::ExtractOfferDetails)
                    .collect())
            },
        )
    }
    pub(super) fn on_extracted(
        &self,
        envelope: &EventEnvelope<OfferDetailsExtracted>,
    ) -> Result<Vec<ApplicationCommand>, String> {
        self.apply(
            envelope,
            envelope.event.fetched.retailer_code,
            |record, conn, cause, event_id| {
                let Some((index, product)) = record.payload.locate_derived(cause, true) else {
                    return Ok(vec![]);
                };
                let Some(reference) =
                    record
                        .payload
                        .accept_extracted(index, &product, event_id, &envelope.event)?
                else {
                    return Ok(vec![]);
                };
                let canonical: OfferDetailsExtracted = load_result(conn, reference.event)?;
                Ok(structuralize(&canonical)
                    .into_iter()
                    .map(ApplicationCommand::StructuralizeOfferDetails)
                    .collect())
            },
        )
    }
    pub(super) fn on_structuralized(
        &self,
        envelope: &EventEnvelope<OfferDetailsStructuralized>,
    ) -> Result<Vec<ApplicationCommand>, String> {
        self.apply(
            envelope,
            envelope.event.extracted.fetched.retailer_code,
            |record, _, cause, event_id| {
                let Some((index, product)) = record.payload.locate_derived(cause, false) else {
                    return Ok(vec![]);
                };
                record
                    .payload
                    .accept_structuralized(index, &product, event_id, &envelope.event)?;
                Ok(vec![])
            },
        )
    }
}
