use super::{command::FetchProductCatalogIdentifiers, event::ProductCatalogIdsRetrieved};
use crate::initial_offer_fill::acquire_storefront_token::prelude::StorefrontTokenAcquired;
use crate::platform::bus::KeyedCommand;
use crate::platform::bus::stable_command_id;
use retailer_sauce::shopify::initial_fill::StorefrontToken;
use shared::retailer::RetailerCode;
use uuid::Uuid;

fn keyed(command: FetchProductCatalogIdentifiers) -> KeyedCommand<FetchProductCatalogIdentifiers> {
    // JSON distinguishes the initial null cursor from any literal cursor string.
    let key = serde_json::to_vec(&("FetchProductCatalogIdentifiers", &command.after))
        .expect("strings serialize");
    let key = if let Some(context) = &command.context {
        serde_json::to_vec(&(
            "FetchProductCatalogIdentifiers",
            &context.country,
            &context.language,
            &command.after,
        ))
        .expect("context serializes")
    } else {
        key
    };
    KeyedCommand {
        id: stable_command_id(&command.run_id, &key),
        run_id: command.run_id,
        command,
    }
}
/// One first catalog page per Retailer supported locale, in declaration order.
pub fn initial(
    run_id: Uuid,
    retailer_code: RetailerCode,
    storefront_token: Option<StorefrontToken>,
) -> Result<Vec<KeyedCommand<FetchProductCatalogIdentifiers>>, String> {
    let retailer = retailers_wip::retailers::get_retailer(retailer_code)
        .ok_or("retailer inventory is missing")?;
    Ok(retailer
        .initial_fill_contexts()?
        .into_iter()
        .map(|context| {
            keyed(FetchProductCatalogIdentifiers {
                context: Some(context),
                run_id,
                retailer_code,
                storefront_token: storefront_token.clone(),
                after: None,
            })
        })
        .collect())
}
pub fn acquired(
    event: &StorefrontTokenAcquired,
) -> Vec<KeyedCommand<FetchProductCatalogIdentifiers>> {
    // The API validated the same inventory before queueing the run.
    initial(
        event.run_id,
        event.retailer_code,
        event.storefront_token.clone(),
    )
    .unwrap_or_else(|error| {
        tracing::error!(run_id=%event.run_id, retailer=%event.retailer_code.as_str(), %error, "initial fill has no catalog contexts");
        Vec::new()
    })
}
pub fn next_batch(
    event: &ProductCatalogIdsRetrieved,
) -> Vec<KeyedCommand<FetchProductCatalogIdentifiers>> {
    event
        .next_cursor
        .as_ref()
        .map(|cursor| {
            keyed(FetchProductCatalogIdentifiers {
                context: event.context.clone(),
                run_id: event.run_id,
                retailer_code: event.retailer_code,
                storefront_token: event.storefront_token.clone(),
                after: Some(cursor.clone()),
            })
        })
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initial_offer_fill::tests::token_value;

    fn token_event(retailer_code: RetailerCode) -> StorefrontTokenAcquired {
        StorefrontTokenAcquired {
            run_id: Uuid::now_v7(),
            retailer_code,
            storefront_token: Some(token_value()),
            url: "https://example.test".into(),
            acquired_at: chrono::Utc::now(),
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
            if retailer_data_extractor::initial_fill_integration_for(code).is_none() {
                continue;
            }
            let event = token_event(code);
            let commands = acquired(&event);
            let expected = retailers_wip::retailers::get_retailer(code)
                .unwrap()
                .initial_fill_contexts()
                .unwrap();
            assert_eq!(
                commands
                    .iter()
                    .map(|c| c.command.context.clone().unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                commands.iter().map(|c| c.id).collect::<Vec<_>>(),
                acquired(&event).iter().map(|c| c.id).collect::<Vec<_>>()
            );
            for command in &commands {
                assert_eq!(command.command.retailer_code, code);
                assert_eq!(command.command.storefront_token, event.storefront_token);
                assert_eq!(command.command.after, None);
            }
            count += 1;
        }
        assert_eq!(count, 48);
    }

    #[test]
    fn ankerca_crawls_both_locales_and_ankerau_one() {
        let pairs = |code| requested(&acquired(&token_event(code)));
        assert_eq!(
            pairs(RetailerCode::AnkerCa),
            [("CA".into(), "EN".into()), ("CA".into(), "FR".into())]
        );
        assert_eq!(pairs(RetailerCode::AnkerAu), [("AU".into(), "EN".into())]);
    }

    #[test]
    fn contexts_keep_replay_safe_catalog_and_detail_identities() {
        let event = token_event(RetailerCode::AnkerCa);
        let commands = acquired(&event);
        assert_eq!(
            commands[0].id,
            stable_command_id(
                &event.run_id,
                br#"["FetchProductCatalogIdentifiers","CA","EN",null]"#
            )
        );
        assert_ne!(commands[0].id, commands[1].id);
        let mut ids = Vec::new();
        for command in commands {
            let event = ProductCatalogIdsRetrieved {
                context: command.command.context,
                run_id: event.run_id,
                retailer_code: event.retailer_code,
                storefront_token: event.storefront_token.clone(),
                after: None,
                product_ids: vec!["gid://shopify/Product/1".into()],
                next_cursor: Some("next".into()),
                retrieved_at: chrono::Utc::now(),
            };
            let next = next_batch(&event);
            assert_eq!(next[0].command.context, event.context);
            assert_eq!(next[0].command.storefront_token, event.storefront_token);
            let details = crate::initial_offer_fill::offer_details::reacts_to::on(&event);
            assert_eq!(details[0].command.context, event.context);
            let language = &event.context.as_ref().unwrap().language;
            let historical_detail_key = serde_json::to_vec(&(
                "FetchOfferDetails",
                "AnkerCa",
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
