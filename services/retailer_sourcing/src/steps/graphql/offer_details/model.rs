use crate::platform::bus::{Command, Event};
use crate::platform::object_storage::OfferStorageMetadata;
use chrono::{DateTime, Utc};
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::{MarketContext, StorefrontToken};
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;

/// The source and input needed to fetch offer details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SourceType {
    Html { url: String },
    GraphQl { product_id: String },
}

/// Fetches all selected offer details from the given source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOfferDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<RequestedContext>,
    pub storefront_token: Option<StorefrontToken>,
    pub retailer_code: RetailerCode,
    pub source: SourceType,
}

impl Command for FetchOfferDetails {
    const NAME: &'static str = "FetchOfferDetails";
}

/// Records the retailer and source used to fetch offer details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferDetailsFetched {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<MarketContext>,
    pub retailer_code: RetailerCode,
    pub source: SourceType,
    pub fetched_at: DateTime<Utc>,
    pub storage: OfferStorageMetadata,
}

impl Event for OfferDetailsFetched {
    const NAME: &'static str = "OfferDetailsFetched";
}
