//! Step that fetches offer details.

use crate::platform::bus::{Command, Event, Handle};
use crate::v2::prelude::storefront_engine;
use chrono::DateTime;
use chrono::Utc;
use retailer_sauce::initial_fill::FetchedDetails;
use retailer_sauce::initial_fill::LocaleInitialFillIntegration;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::MarketContext;
use retailer_sauce::prelude::{Http, StorefrontToken};
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use tracing::info;

use crate::platform::object_storage::{ObjectStorage, OfferStorageMetadata};

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

impl FetchOfferDetails {
    pub fn new(
        retailer_code: RetailerCode,
        context: Option<RequestedContext>,
        storefront_token: Option<StorefrontToken>,
        source: SourceType,
    ) -> Self {
        Self {
            context,
            storefront_token,
            retailer_code,
            source,
        }
    }
}

impl Command for FetchOfferDetails {
    const NAME: &'static str = "FetchOfferDetails";
}

pub const BUCKET: &str = "offer_details_raw";
pub trait Store: Send + Sync + 'static {
    fn store(&self, body: &str, fetched_at: DateTime<Utc>) -> Result<OfferStorageMetadata, String>;
}
impl Store for ObjectStorage {
    fn store(&self, body: &str, fetched_at: DateTime<Utc>) -> Result<OfferStorageMetadata, String> {
        let mut metadata = self.create_offer(BUCKET, body).map_err(|e| e.to_string())?;
        metadata.scraped_at = Some(fetched_at);
        Ok(metadata)
    }
}
pub struct Handler<F, S> {
    pub fetcher: F,
    pub store: S,
}
impl<F: Http + Send + Sync + 'static, S: Store> Handle for Handler<F, S> {
    type Command = FetchOfferDetails;
    type Event = OfferDetailsFetched;
    type Error = String;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, String> {
        let SourceType::GraphQl { product_id } = &command.source else {
            return Err("HTML initial-fill details are not implemented".into());
        };
        let details = fetch_for_locale(
            &self.fetcher,
            command.retailer_code,
            command.storefront_token.as_ref(),
            product_id,
            command.context.as_ref(),
        )?;
        let fetched_at = Utc::now();
        let storage = self.store.store(&details.body, fetched_at)?;
        info!(context = ?command.context, product_id, object_storage_id = %storage.object_storage_id, "complete offer details stored");
        let fetched = OfferDetailsFetched {
            context: details.context,
            retailer_code: command.retailer_code,
            source: command.source,
            fetched_at,
            storage,
        };

        Ok(vec![fetched])
    }
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

pub fn fetch_for_locale<F: Http>(
    fetcher: &F,
    retailer: RetailerCode,
    token: Option<&StorefrontToken>,
    id: &str,
    context: Option<&RequestedContext>,
) -> Result<FetchedDetails, String> {
    storefront_engine(retailer)?.details_for_locale(fetcher, token, context, id)
}

#[cfg(test)]
pub fn fetch_in_context<F: Http>(
    fetcher: &F,
    retailer: RetailerCode,
    token: Option<&StorefrontToken>,
    id: &str,
    context: Option<&MarketContext>,
) -> Result<String, String> {
    use retailer_sauce::prelude::InitialFillIntegration;

    storefront_engine(retailer)?.details(fetcher, token, context, id)
}
