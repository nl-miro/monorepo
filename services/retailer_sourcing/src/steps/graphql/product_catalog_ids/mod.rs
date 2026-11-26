//! Collect all product identifiers for one configured retailer locale.

use crate::platform::bus::{Command, Event, Handle};
use crate::v2::prelude::storefront_engine;
use chrono::DateTime;
use chrono::Utc;
use retailer_sauce::initial_fill::LocaleInitialFillIntegration;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::{Http, StorefrontToken};
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use std::collections::HashSet;
use tracing::info;

/// Collect every catalog identifier of one locale within an initial-fill run.
/// Commands queued by the former per-batch continuation also carried `after`;
/// serde ignores it, so they walk the whole catalog from the start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchProductCatalogIdentifiers {
    /// Locale and consistency checks carried through the traversal. Older queued
    /// commands may omit this field, so deserialization still accepts `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<RequestedContext>,
    pub retailer_code: RetailerCode,
    pub storefront_token: Option<StorefrontToken>,
}

impl FetchProductCatalogIdentifiers {
    pub fn new(
        retailer_code: RetailerCode,
        context: RequestedContext,
        storefront_token: Option<StorefrontToken>,
    ) -> Self {
        Self {
            retailer_code,
            context: Some(context),
            storefront_token,
        }
    }
}

impl Command for FetchProductCatalogIdentifiers {
    const NAME: &'static str = "FetchProductCatalogIdentifiers";
}

pub struct Handler<F> {
    pub fetcher: F,
}
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
                &self.fetcher,
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
            info!(retailer = ?command.retailer_code, context = ?context, loop_count, count, total = product_ids.len(), "catalog batch retrieved");
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

/// Every product ID of one locale's catalog.
/// Historical per-batch events also carried `after` and `next_cursor`; serde
/// ignores those fields when reading them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductCatalogIdsRetrieved {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<RequestedContext>,
    pub retailer_code: RetailerCode,
    pub storefront_token: Option<StorefrontToken>,
    /// Number of unique product IDs found. Historical events read as 0.
    #[serde(default)]
    pub product_count: usize,
    /// Number of catalog batches fetched to walk the whole catalog. Historical
    /// events read as 0.
    #[serde(default)]
    pub loop_count: usize,
    pub product_ids: Vec<String>,
    pub retrieved_at: DateTime<Utc>,
}
impl Event for ProductCatalogIdsRetrieved {
    const NAME: &'static str = "ProductCatalogIdsRetrieved";
}
