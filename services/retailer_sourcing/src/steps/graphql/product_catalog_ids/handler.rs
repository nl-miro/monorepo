use super::super::storefront_engine;
use super::model::{FetchProductCatalogIdentifiers, ProductCatalogIdsRetrieved};
use crate::platform::bus::Handle;
use chrono::Utc;
use retailer_sauce::initial_fill::LocaleInitialFillIntegration;
use retailer_sauce::prelude::Http;
use std::collections::HashSet;
use tracing::info;

pub struct Handler<F>(pub F);
impl<F: Http + Send + Sync + 'static> Handle for Handler<F> {
    type Command = FetchProductCatalogIdentifiers;
    type Event = ProductCatalogIdsRetrieved;
    type Error = String;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, String> {
        // The storefront engine owns API requests, pagination within a batch and
        // response validation. Propagate failures so the queue can retry them.
        let engine = storefront_engine(command.retailer_code)?;
        let mut context = command.context;
        let mut after = None;
        let mut seen_cursors = HashSet::new();
        let mut seen_ids = HashSet::new();
        let mut product_ids = Vec::new();
        let mut loop_count = 0;
        loop {
            loop_count += 1;
            let batch = engine.catalog_for_locale(
                &self.0,
                command.storefront_token.as_ref(),
                context.as_ref(),
                after.take(),
            )?;
            // Carry the integration's resolved context forward, including any
            // observed defaults needed to check later batches and details.
            context = batch.context;
            let count = batch.batch.product_ids.len();
            // Integrations deduplicate within a batch; keep first-seen order
            // across batches too.
            for id in batch.batch.product_ids {
                if seen_ids.insert(id.clone()) {
                    product_ids.push(id);
                }
            }
            info!(context = ?context, loop_count, count, total = product_ids.len(), "catalog batch retrieved");
            match batch.batch.next_cursor {
                None => break,
                // Integrations only detect stalled cursors within a batch; a
                // cycle across batches would otherwise loop forever.
                Some(cursor) if !seen_cursors.insert(cursor.clone()) => {
                    return Err(format!("catalog cursor {cursor} repeated"));
                }
                Some(cursor) => after = Some(cursor),
            }
        }
        let retrieved = ProductCatalogIdsRetrieved {
            context,

            retailer_code: command.retailer_code,
            storefront_token: command.storefront_token,
            product_count: product_ids.len(),
            loop_count,
            product_ids,
            retrieved_at: Utc::now(),
        };

        Ok(vec![retrieved])
    }
}
