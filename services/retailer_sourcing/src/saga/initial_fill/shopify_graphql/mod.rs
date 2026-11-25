//! Shopify GraphQL initial-fill saga.

mod entity;
mod listener;
mod model;

pub(crate) use entity::ShopifyGraphQlInitialFillSaga;
pub(super) use listener::Listener;
pub(crate) use model::{SagaId, UuidV7};

#[cfg(test)]
mod tests {
    use super::entity::StartingCommand;
    use super::{SagaId, ShopifyGraphQlInitialFillSaga};
    use crate::platform::bus::{Command, Event, EventCommand, EventContext};
    use crate::platform::object_storage::OfferStorageMetadata;
    use crate::steps::prelude::{
        ExtractOfferDetails, InitialFillRequested, OfferDetailsExtracted, OfferDetailsFetched,
        SourceType, StorefrontTokenAcquired, StructuralizeOfferDetails,
    };
    use chrono::Utc;
    use retailer_data_extractor::get_storefront_config;
    use retailer_sauce::prelude::{StorefrontStrategy, StorefrontToken};
    use serde_json::{Value, from_str, json};
    use shared::retailer::RetailerCode;
    use uuid::Uuid;

    #[test]
    fn starting_envelopes_use_the_saga_id_as_correlation_and_event_id_as_causation() {
        let id = SagaId::new();
        let request_id = Uuid::now_v7();
        let context = EventContext {
            event_id: Uuid::now_v7(),
            correlation_id: Some(request_id),
            causation_id: None,
            source: None,
        };
        for retailer in [RetailerCode::AnkerAu, RetailerCode::UgreenEu] {
            let saga = ShopifyGraphQlInitialFillSaga::new(id, request_id, retailer);
            let commands = saga.on_initial_fill_requested().unwrap();
            assert!(!commands.is_empty());
            for command in commands {
                let envelope = command
                    .into_envelope(saga.saga_id().into(), InitialFillRequested::NAME, &context)
                    .unwrap();
                let metadata = envelope.metadata.unwrap();
                assert_eq!(metadata.correlation_id, Some(Uuid::from(id)));
                assert_eq!(metadata.causation_id, Some(context.event_id));
            }
        }
    }

    #[test]
    fn derived_commands_carry_offer_details_and_use_the_source_object_for_identity() {
        let saga_id = SagaId::new();
        let retailer = RetailerCode::AnkerAu;
        let saga = ShopifyGraphQlInitialFillSaga::new(saga_id, Uuid::now_v7(), retailer);
        let raw = OfferStorageMetadata {
            scraped_at: None,
            content_size: 10,
            object_storage_id: Uuid::now_v7(),
            external_id: Uuid::now_v7(),
        };
        let fetched = OfferDetailsFetched {
            context: None,
            retailer_code: retailer,
            fetched_at: Utc::now(),
            storage: raw,
            source: SourceType::GraphQl {
                product_id: "gid://shopify/Product/1".into(),
            },
        };
        let extracted = OfferDetailsExtracted {
            retailer_code: fetched.retailer_code,
            source: fetched.source.clone(),
            storage: OfferStorageMetadata {
                object_storage_id: Uuid::now_v7(),
                external_id: raw.object_storage_id,
                ..raw
            },
            fetched_at: fetched.fetched_at,
        };
        for (command, event_name, name, payload, source_id) in [
            (
                saga.on_offer_details_fetched(fetched.clone()),
                OfferDetailsFetched::NAME,
                ExtractOfferDetails::NAME,
                json!({
                    "retailer_code": fetched.retailer_code,
                    "source": fetched.source,
                    "storage": fetched.storage,
                    "fetched_at": fetched.fetched_at,
                }),
                raw.object_storage_id,
            ),
            (
                saga.on_offer_details_extracted(extracted.clone()),
                OfferDetailsExtracted::NAME,
                StructuralizeOfferDetails::NAME,
                json!(extracted),
                extracted.storage.object_storage_id,
            ),
        ] {
            let context = EventContext {
                event_id: Uuid::now_v7(),
                correlation_id: Some(saga_id.into()),
                causation_id: None,
                source: None,
            };
            let envelope = command
                .into_envelope(saga_id.into(), event_name, &context)
                .unwrap();
            let meta = envelope.metadata.unwrap();
            assert_eq!(envelope.command.command_type, name);
            assert_eq!(
                from_str::<Value>(&envelope.command.payload).unwrap(),
                payload
            );
            assert_eq!(meta.command_id, Uuid::new_v5(&source_id, name.as_bytes()));
            assert_eq!(meta.causation_id, Some(context.event_id));
            assert_eq!(meta.correlation_id, Some(saga_id.into()));
            assert_eq!(meta.source, Some(format!("event:{event_name}")));
        }
    }

    #[test]
    fn starts_each_configured_storefront_with_its_token_policy_and_locales() {
        let mut token_stores = Vec::new();
        let mut tokenless_stores = Vec::new();
        for retailer in RetailerCode::ALL {
            let Some(config) = get_storefront_config(retailer) else {
                continue;
            };
            if config.access.preferred != StorefrontStrategy::ShopifyGraphQl {
                continue;
            }
            let id = SagaId::new();
            let saga = ShopifyGraphQlInitialFillSaga::new(id, Uuid::now_v7(), retailer);
            let initial = saga.on_initial_fill_requested().unwrap();
            let (commands, token) = if config.attempts_token() {
                token_stores.push(retailer);
                let [StartingCommand::Token(command)] = initial.as_slice() else {
                    panic!("expected one token command for {retailer:?}");
                };
                assert_eq!(command.retailer_code, retailer);
                assert_eq!(command.url, config.storefront_url());
                let token = Some(StorefrontToken::new("test-token".into()).unwrap());
                let event = StorefrontTokenAcquired {
                    retailer_code: retailer,
                    storefront_token: token.clone(),
                    url: command.url.clone(),
                    acquired_at: Utc::now(),
                };
                (saga.on_storefront_token_acquired(event).unwrap(), token)
            } else {
                tokenless_stores.push(retailer);
                (initial, None)
            };

            let locales = config.initial_fill_contexts().unwrap();
            assert_eq!(commands.len(), locales.len());
            for (queued, locale) in commands.into_iter().zip(locales) {
                let StartingCommand::CatalogProductIds(command) = queued else {
                    panic!("expected a catalog command for {retailer:?}");
                };
                assert_eq!(command.retailer_code, retailer);
                assert_eq!(command.context, Some(locale));
                assert_eq!(command.storefront_token, token);
            }
        }
        assert!(token_stores.contains(&RetailerCode::AnkerAu));
        assert!(tokenless_stores.contains(&RetailerCode::UgreenEu));
    }
}
