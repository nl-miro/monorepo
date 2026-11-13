//! Snapshot inventory supported locales and fan out replay-safe catalog commands.
use super::product_catalog_ids::command::FetchProductCatalogIdentifiers;
use crate::platform::{
    Platform,
    bus::{Bus, Command, Event, Handle, KeyedCommand, stable_command_id},
};
use retailers_wip::initial_fill::RequestedContext;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverStorefrontMarkets {
    pub run_id: Uuid,
    #[serde(default)]
    pub storefront_token: Option<retailer_sauce::shopify::initial_fill::StorefrontToken>,
    pub retailer_code: RetailerCode,
}
impl Command for DiscoverStorefrontMarkets {
    const NAME: &'static str = "DiscoverStorefrontMarkets";
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorefrontMarketsDiscovered {
    pub run_id: Uuid,
    #[serde(default)]
    pub storefront_token: Option<retailer_sauce::shopify::initial_fill::StorefrontToken>,
    pub retailer_code: RetailerCode,
    pub contexts: Vec<RequestedContext>,
}
impl Event for StorefrontMarketsDiscovered {
    const NAME: &'static str = "StorefrontMarketsDiscovered";
}
struct Handler;
impl Handle for Handler {
    type Command = DiscoverStorefrontMarkets;
    type Event = StorefrontMarketsDiscovered;
    type Error = String;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, String> {
        let _integration =
            retailer_data_extractor::initial_fill_integration_for(command.retailer_code)
                .ok_or("initial fill is not configured")?;
        let retailer = retailers_wip::retailers::get_retailer(command.retailer_code)
            .ok_or("retailer inventory is missing")?;
        let contexts = retailer.initial_fill_contexts()?;
        Ok(vec![StorefrontMarketsDiscovered {
            storefront_token: command.storefront_token,
            run_id: command.run_id,
            retailer_code: command.retailer_code,
            contexts,
        }])
    }
}
fn on(event: &StorefrontMarketsDiscovered) -> Vec<KeyedCommand<FetchProductCatalogIdentifiers>> {
    event
        .contexts
        .iter()
        .map(|context| {
            let key = serde_json::to_vec(&(
                "FetchProductCatalogIdentifiers",
                &context.country,
                &context.language,
                Option::<String>::None,
            ))
            .expect("context serializes");
            KeyedCommand {
                id: stable_command_id(&event.run_id, &key),
                run_id: event.run_id,
                command: FetchProductCatalogIdentifiers {
                    run_id: event.run_id,
                    retailer_code: event.retailer_code,
                    storefront_token: event.storefront_token.clone(),
                    after: None,
                    context: Some(context.clone()),
                },
            }
        })
        .collect()
}
pub fn register(bus: &mut Bus, _platform: &Platform) {
    register_inventory(bus);
}

pub(super) fn register_inventory(bus: &mut Bus) {
    bus.handle(Handler);
    bus.on_many_keyed(on);
    bus.on_many_keyed(acquired);
}

fn acquired(
    event: &super::acquire_storefront_token::prelude::StorefrontTokenAcquired,
) -> Vec<KeyedCommand<DiscoverStorefrontMarkets>> {
    if !retailer_data_extractor::all_market_contexts(event.retailer_code) {
        return Vec::new();
    }
    vec![KeyedCommand {
        id: stable_command_id(&event.run_id, b"DiscoverStorefrontMarkets"),
        run_id: event.run_id,
        command: DiscoverStorefrontMarkets {
            run_id: event.run_id,
            retailer_code: event.retailer_code,
            storefront_token: event.storefront_token.clone(),
        },
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_anker_token_event_starts_discovery_instead_of_fixed_catalog() {
        for retailer_code in shared::retailer::RetailerCode::ALL {
            if !retailer_code.as_str().starts_with("anker")
                || retailer_data_extractor::initial_fill_integration_for(retailer_code).is_none()
            {
                continue;
            }
            let event = super::super::acquire_storefront_token::prelude::StorefrontTokenAcquired {
                run_id: Uuid::now_v7(),
                retailer_code,
                storefront_token: None,
                url: "https://example.test".into(),
                acquired_at: chrono::Utc::now(),
            };
            assert!(super::super::product_catalog_ids::reacts_to::acquired(&event).is_empty());
            let commands = acquired(&event);
            assert_eq!(commands.len(), 1);
            assert_eq!(commands[0].id, acquired(&event)[0].id);
            assert_eq!(commands[0].command.retailer_code, retailer_code);
        }
    }
    #[test]
    fn discovery_snapshots_all_supported_inventory_without_http() {
        let mut count = 0;
        for code in RetailerCode::ALL {
            if retailer_data_extractor::initial_fill_integration_for(code).is_none() {
                continue;
            }
            let event = Handler
                .handle(DiscoverStorefrontMarkets {
                    run_id: Uuid::now_v7(),
                    storefront_token: None,
                    retailer_code: code,
                })
                .unwrap()
                .remove(0);
            let expected = retailers_wip::retailers::get_retailer(code)
                .unwrap()
                .initial_fill_contexts()
                .unwrap();
            assert_eq!(event.contexts, expected);
            let replay: StorefrontMarketsDiscovered =
                serde_json::from_value(serde_json::to_value(&event).unwrap()).unwrap();
            assert_eq!(
                on(&event).iter().map(|c| c.id).collect::<Vec<_>>(),
                on(&replay).iter().map(|c| c.id).collect::<Vec<_>>()
            );
            count += 1;
        }
        assert_eq!(count, 48);
    }
    #[test]
    fn contexts_have_distinct_replay_safe_catalog_and_detail_commands() {
        let event = StorefrontMarketsDiscovered {
            storefront_token: None,
            run_id: Uuid::now_v7(),
            retailer_code: RetailerCode::UgreenCa,
            contexts: vec![
                RequestedContext {
                    country: Some("CA".into()),
                    language: "EN".into(),
                    currency: Some("CAD".into()),
                    observed_country: None,
                },
                RequestedContext {
                    country: Some("CA".into()),
                    language: "FR".into(),
                    currency: Some("CAD".into()),
                    observed_country: None,
                },
            ],
        };
        let commands = on(&event);
        assert_eq!(commands[0].id, on(&event)[0].id);
        let historical_key = br#"["FetchProductCatalogIdentifiers","CA","EN",null]"#;
        assert_eq!(
            commands[0].id,
            stable_command_id(&event.run_id, historical_key)
        );
        assert_ne!(commands[0].id, commands[1].id);
        let mut ids = Vec::new();
        for command in commands {
            let event = super::super::product_catalog_ids::event::ProductCatalogIdsRetrieved {
                context: command.command.context,
                run_id: event.run_id,
                retailer_code: event.retailer_code,
                storefront_token: event.storefront_token.clone(),
                after: None,
                product_ids: vec!["gid://shopify/Product/1".into()],
                next_cursor: Some("next".into()),
                retrieved_at: chrono::Utc::now(),
            };
            let next = super::super::product_catalog_ids::reacts_to::next_batch(&event);
            assert_eq!(next[0].command.context, event.context);
            let details = super::super::offer_details::reacts_to::on(&event);
            assert_eq!(details[0].command.context, event.context);
            let language = &event.context.as_ref().unwrap().language;
            let historical_detail_key = serde_json::to_vec(&(
                "FetchOfferDetails",
                "UgreenCa",
                "CA",
                language,
                "gid://shopify/Product/1",
            ))
            .unwrap();
            assert_eq!(
                details[0].id,
                stable_command_id(&event.run_id, &historical_detail_key)
            );
            ids.push(details[0].id);
        }
        assert_ne!(ids[0], ids[1]);
    }
}

#[cfg(test)]
mod token_tests {
    use super::super::tests::token_value;
    use super::*;
    #[test]
    fn acquired_token_survives_discovery_and_catalog_fanout() {
        let event = super::super::acquire_storefront_token::prelude::StorefrontTokenAcquired {
            run_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            storefront_token: Some(token_value()),
            url: "https://www.anker.com/au".into(),
            acquired_at: chrono::Utc::now(),
        };
        let command = acquired(&event).remove(0).command;
        assert_eq!(command.storefront_token, event.storefront_token);
        let handler = Handler;
        let result = handler.handle(command).unwrap().remove(0);
        for command in on(&result) {
            assert_eq!(command.command.storefront_token, event.storefront_token);
        }
    }
}
