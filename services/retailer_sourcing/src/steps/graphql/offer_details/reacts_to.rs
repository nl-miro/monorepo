use super::super::product_catalog_ids::ProductCatalogIdsRetrieved;
use super::{FetchOfferDetails, SourceType};
use crate::platform::bus::{KeyedCommand, stable_command_id};
use serde_json::to_vec;
use uuid::Uuid;

pub fn fetch_details_for_catalog_products(
    namespace: Uuid,
    event: &ProductCatalogIdsRetrieved,
) -> Vec<KeyedCommand<FetchOfferDetails>> {
    event
        .product_ids
        .iter()
        .map(|id| {
            // Preserve the original identity bytes when replaying queued events;
            // payload serialization now uses lowercase retailer slugs (ADR 49).
            let key = to_vec(&(
                "FetchOfferDetails",
                format!("{:?}", event.retailer_code),
                id,
            ))
            .expect("IDs serialize");

            let key = if let Some(context) = &event.context {
                to_vec(&(
                    "FetchOfferDetails",
                    format!("{:?}", event.retailer_code),
                    &context.country,
                    &context.language,
                    id,
                ))
                .expect("context serializes")
            } else {
                key
            };

            KeyedCommand {
                id: stable_command_id(&namespace, &key),

                command: FetchOfferDetails {
                    context: event.context.clone(),

                    retailer_code: event.retailer_code,
                    storefront_token: event.storefront_token.clone(),
                    source: SourceType::GraphQl {
                        product_id: id.clone(),
                    },
                },
            }
        })
        .collect()
}
