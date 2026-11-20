//! Step that fetches offer details.

mod handler;
mod model;
pub(crate) mod prelude;
#[cfg(test)]
mod reacts_to;

pub use self::handler::Handler;
#[cfg(test)]
pub use self::handler::{BUCKET, Store};
pub use self::model::{FetchOfferDetails, OfferDetailsFetched, SourceType};
#[cfg(test)]
pub use self::reacts_to::fetch_details_for_catalog_products;

#[cfg(test)]
mod tests {
    use super::super::product_catalog_ids::{ProductCatalogIdsRetrieved, initial};
    use super::*;
    use crate::platform::bus::stable_command_id;
    use crate::v2::prelude::InitialFillContexts;
    use chrono::Utc;
    use retailer_data_extractor::get_storefront_config;
    use serde_json::to_vec;
    use shared::retailer::RetailerCode;
    use std::collections::HashSet;
    use uuid::Uuid;

    #[test]
    fn catalog_products_keep_context_and_historical_command_ids() {
        let contexts = get_storefront_config(RetailerCode::AnkerCa)
            .unwrap()
            .initial_fill_contexts()
            .unwrap();
        let catalogs = initial(Uuid::nil(), RetailerCode::AnkerCa, None, &contexts);
        let mut ids = Vec::new();
        for catalog in catalogs {
            let event = ProductCatalogIdsRetrieved {
                context: catalog.command.context,
                retailer_code: RetailerCode::AnkerCa,
                storefront_token: None,
                product_count: 2,
                loop_count: 1,
                product_ids: vec![
                    "gid://shopify/Product/1".into(),
                    "gid://shopify/Product/2".into(),
                ],
                retrieved_at: Utc::now(),
            };
            let commands = fetch_details_for_catalog_products(Uuid::nil(), &event);
            assert_eq!(commands.len(), 2);
            for (command, product_id) in commands.iter().zip(&event.product_ids) {
                let context = event.context.as_ref().unwrap();
                let key = to_vec(&(
                    "FetchOfferDetails",
                    "AnkerCa",
                    &context.country,
                    &context.language,
                    product_id,
                ))
                .unwrap();
                assert_eq!(command.id, stable_command_id(&Uuid::nil(), &key));
                assert_eq!(command.command.context, event.context);
                assert_eq!(
                    command.command.source,
                    SourceType::GraphQl {
                        product_id: product_id.clone()
                    }
                );
                ids.push(command.id);
            }
            let empty = ProductCatalogIdsRetrieved {
                product_ids: Vec::new(),
                ..event
            };
            assert!(fetch_details_for_catalog_products(Uuid::nil(), &empty).is_empty());
        }
        assert_eq!(ids.iter().collect::<HashSet<_>>().len(), 4);
    }
}
