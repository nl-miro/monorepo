use super::model::FetchProductCatalogIdentifiers;
use crate::platform::bus::{KeyedCommand, stable_command_id};
#[cfg(test)]
use crate::v2::prelude::InitialFillContexts;
#[cfg(test)]
use retailer_data_extractor::get_storefront_config;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::StorefrontToken;
use serde_json::to_vec;
use shared::retailer::RetailerCode;
use uuid::Uuid;

/// Replaying an event must produce the same command ID within its run, so queue
/// deduplication can suppress duplicate work. A new run gets new IDs. The key
/// holds the requested country/language, not observed facts that may change
/// after a fetch. The trailing `null` is the start cursor of the former
/// per-batch commands, kept so command IDs stay unchanged (ADR 74).
pub(crate) fn keyed(
    namespace: Uuid,
    context: &RequestedContext,
    command: FetchProductCatalogIdentifiers,
) -> KeyedCommand<FetchProductCatalogIdentifiers> {
    let key = to_vec(&(
        "FetchProductCatalogIdentifiers",
        &context.country,
        &context.language,
        None::<String>,
    ))
    .expect("context serializes");
    KeyedCommand {
        id: stable_command_id(&namespace, &key),

        command,
    }
}
/// Build catalog work from an immutable context snapshot.
pub(crate) fn initial(
    namespace: Uuid,
    retailer_code: RetailerCode,
    storefront_token: Option<StorefrontToken>,
    contexts: &[RequestedContext],
) -> Vec<KeyedCommand<FetchProductCatalogIdentifiers>> {
    contexts
        .iter()
        .map(|context| {
            keyed(
                namespace,
                context,
                FetchProductCatalogIdentifiers {
                    retailer_code,
                    context: Some(context.clone()),
                    storefront_token: storefront_token.clone(),
                },
            )
        })
        .collect()
}

#[cfg(test)]
pub fn start_catalog_for_locales(
    namespace: Uuid,
    event: &super::super::acquire_storefront_token::StorefrontTokenAcquired,
) -> Vec<KeyedCommand<FetchProductCatalogIdentifiers>> {
    let config = get_storefront_config(event.retailer_code).unwrap();
    initial(
        namespace,
        event.retailer_code,
        event.storefront_token.clone(),
        &config.initial_fill_contexts().unwrap(),
    )
}
