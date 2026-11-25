//! Collect product identifiers for each configured retailer locale.
//!
//! `initial` starts one catalog traversal per locale. The handler walks the
//! whole catalog for that locale, batch by batch, and emits a single
//! `ProductCatalogIdsRetrieved` with every product ID; a reaction uses it to
//! queue product details. A failed batch retries the whole traversal; detail
//! command IDs are stable per product, so the queue drops repeated work.

mod handler;
mod model;
pub(crate) mod prelude;
mod reacts_to;

pub use self::handler::Handler;
pub use self::model::{FetchProductCatalogIdentifiers, ProductCatalogIdsRetrieved};

pub(crate) use self::reacts_to::initial;
#[cfg(test)]
pub use self::reacts_to::start_catalog_for_locales;

#[cfg(test)]
mod tests {
    use super::super::acquire_storefront_token::StorefrontTokenAcquired;
    use super::super::offer_details::fetch_details_for_catalog_products;
    use super::{
        FetchProductCatalogIdentifiers, ProductCatalogIdsRetrieved, start_catalog_for_locales,
    };
    use crate::platform::bus::{KeyedCommand, stable_command_id};
    use crate::v2::prelude::InitialFillContexts;
    use chrono::Utc;
    use retailer_data_extractor::get_storefront_config;
    use retailer_sauce::prelude::StorefrontToken;
    use serde_json::to_vec;
    use shared::retailer::RetailerCode;
    use uuid::Uuid;

    fn token_value() -> StorefrontToken {
        StorefrontToken::new("synthetic-au-token".into()).unwrap()
    }

    fn token_event(retailer_code: RetailerCode) -> StorefrontTokenAcquired {
        StorefrontTokenAcquired {
            retailer_code,
            storefront_token: Some(token_value()),
            url: "https://example.test".into(),
            acquired_at: Utc::now(),
        }
    }

    fn requested(
        commands: &[KeyedCommand<FetchProductCatalogIdentifiers>],
    ) -> Vec<(String, String)> {
        commands
            .iter()
            .map(|c| {
                let context = c.command.context.as_ref().unwrap();
                (
                    context.country.clone().unwrap_or_default(),
                    context.language.clone(),
                )
            })
            .collect()
    }

    #[test]
    fn token_event_fans_out_one_catalog_command_per_supported_locale() {
        let mut count = 0;
        for code in RetailerCode::ALL {
            let Some(config) = get_storefront_config(code) else {
                continue;
            };
            let event = token_event(code);
            let commands = start_catalog_for_locales(Uuid::nil(), &event);
            let expected = config.initial_fill_contexts().unwrap();
            assert_eq!(
                commands
                    .iter()
                    .map(|c| c.command.context.clone().unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                commands.iter().map(|c| c.id).collect::<Vec<_>>(),
                start_catalog_for_locales(Uuid::nil(), &event)
                    .iter()
                    .map(|c| c.id)
                    .collect::<Vec<_>>()
            );
            for command in &commands {
                assert_eq!(command.command.retailer_code, code);
                assert_eq!(command.command.storefront_token, event.storefront_token);
            }
            count += 1;
        }
        assert_eq!(count, 48);
    }

    #[test]
    fn ankerca_crawls_both_locales_and_ankerau_one() {
        let pairs = |code| requested(&start_catalog_for_locales(Uuid::nil(), &token_event(code)));
        assert_eq!(
            pairs(RetailerCode::AnkerCa),
            [("CA".into(), "EN".into()), ("CA".into(), "FR".into())]
        );
        assert_eq!(pairs(RetailerCode::AnkerAu), [("AU".into(), "EN".into())]);
    }

    #[test]
    fn contexts_keep_replay_safe_catalog_and_detail_identities() {
        let event = token_event(RetailerCode::AnkerCa);
        let commands = start_catalog_for_locales(Uuid::nil(), &event);
        assert_eq!(
            commands[0].id,
            stable_command_id(
                &Uuid::nil(),
                br#"["FetchProductCatalogIdentifiers","CA","EN",null]"#
            )
        );
        assert_ne!(commands[0].id, commands[1].id);
        let mut ids = Vec::new();
        for command in commands {
            let event = ProductCatalogIdsRetrieved {
                context: command.command.context,

                retailer_code: event.retailer_code,
                storefront_token: event.storefront_token.clone(),
                product_count: 1,
                loop_count: 1,
                product_ids: vec!["gid://shopify/Product/1".into()],
                retrieved_at: Utc::now(),
            };
            let details = fetch_details_for_catalog_products(Uuid::nil(), &event);
            assert_eq!(details[0].command.context, event.context);
            let language = &event.context.as_ref().unwrap().language;
            let historical_detail_key = to_vec(&(
                "FetchOfferDetails",
                "AnkerCa",
                "CA",
                language,
                "gid://shopify/Product/1",
            ))
            .unwrap();
            assert_eq!(
                details[0].id,
                stable_command_id(&Uuid::nil(), &historical_detail_key)
            );
            ids.push(details[0].id);
        }
        assert_ne!(ids[0], ids[1]);
    }
}
