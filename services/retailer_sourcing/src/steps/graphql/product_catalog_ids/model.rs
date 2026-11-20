use crate::platform::bus::{Command, Event};
use chrono::{DateTime, Utc};
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::StorefrontToken;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;

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
impl Command for FetchProductCatalogIdentifiers {
    const NAME: &'static str = "FetchProductCatalogIdentifiers";
}

/// Every product ID of one locale's catalog, consumed by the detail-fetch reaction.
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
