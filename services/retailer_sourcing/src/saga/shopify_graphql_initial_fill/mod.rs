//! Shopify GraphQL initial fill saga.

mod listener;
mod model;
#[cfg(test)]
mod postgres_tests;
mod reconciliation;
mod results;
mod service;
#[cfg(test)]
#[path = "transition_tests.rs"]
mod transition_postgres_tests;
mod ui;

use self::listener::Listener;
pub(crate) use self::model::ShopifyGraphQlInitialFillSaga;
pub(crate) use self::reconciliation::reconcile;
pub(crate) use self::ui::prelude::routes;
use crate::platform::bus::{Bus, IntoEventCommand, KeyedCommand};
use crate::platform::db::DbPool;
use crate::saga::saga::Saga;
use crate::saga::saga_repository::SagaRepository;
use crate::steps::graphql::extract_offer_details::ExtractOfferDetails;
use crate::steps::graphql::offer_details::FetchOfferDetails;
use crate::steps::graphql::structuralize_offer_details::StructuralizeOfferDetails;
use crate::steps::prelude::{AcquireStorefrontToken, FetchProductCatalogIdentifiers};
use kernel::prelude::NewCommand;
use serde_json::Error as JsonError;

#[derive(Debug)]
pub(crate) enum ApplicationCommand {
    AcquireStorefrontToken(KeyedCommand<AcquireStorefrontToken>),
    FetchProductCatalogIdentifiers(KeyedCommand<FetchProductCatalogIdentifiers>),
    FetchOfferDetails(KeyedCommand<FetchOfferDetails>),
    ExtractOfferDetails(KeyedCommand<ExtractOfferDetails>),
    StructuralizeOfferDetails(KeyedCommand<StructuralizeOfferDetails>),
}
impl IntoEventCommand for ApplicationCommand {
    fn encode(self) -> Result<KeyedCommand<NewCommand>, JsonError> {
        match self {
            Self::AcquireStorefrontToken(command) => command.encode(),
            Self::FetchProductCatalogIdentifiers(command) => command.encode(),
            Self::FetchOfferDetails(command) => command.encode(),
            Self::ExtractOfferDetails(command) => command.encode(),
            Self::StructuralizeOfferDetails(command) => command.encode(),
        }
    }
}
pub(crate) fn register(bus: &mut Bus, db: &DbPool) {
    let reconciliation_pool = db.clone();
    bus.after_batch(move || {
        reconcile(&reconciliation_pool)
            .map(|_| ())
            .map_err(|error| error.to_string())
    });
    let listener = Listener::new(SagaRepository::new(db.clone()));
    let initial = listener.clone();
    bus.on_commands_envelope(ShopifyGraphQlInitialFillSaga::TYPE, move |event| {
        initial.on_retailer_fill_initialized(event)
    });
    let token = listener.clone();
    bus.on_commands_envelope(ShopifyGraphQlInitialFillSaga::TYPE, move |event| {
        token.on_token(event)
    });
    let catalog = listener.clone();
    bus.on_commands_envelope(ShopifyGraphQlInitialFillSaga::TYPE, move |event| {
        catalog.on_catalog(event)
    });
    let fetched = listener.clone();
    bus.on_commands_envelope(ShopifyGraphQlInitialFillSaga::TYPE, move |event| {
        fetched.on_fetched(event)
    });
    let extracted = listener.clone();
    bus.on_commands_envelope(ShopifyGraphQlInitialFillSaga::TYPE, move |event| {
        extracted.on_extracted(event)
    });
    bus.on_commands_envelope(ShopifyGraphQlInitialFillSaga::TYPE, move |event| {
        listener.on_structuralized(event)
    });
}
