use super::super::storefront_engine;
use super::{FetchOfferDetails, OfferDetailsFetched, SourceType};
use crate::platform::bus::Handle;
use crate::platform::object_storage::{ObjectStorage, OfferStorageMetadata};
use chrono::{DateTime, Utc};
use retailer_sauce::initial_fill::{
    FetchedDetails, LocaleInitialFillIntegration, RequestedContext,
};
use retailer_sauce::prelude::Http;
use retailer_sauce::prelude::StorefrontToken;
use shared::retailer::RetailerCode;
use tracing::info;

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

pub fn fetch_for_locale<F: Http>(
    fetcher: &F,
    retailer: RetailerCode,
    token: Option<&StorefrontToken>,
    id: &str,
    context: Option<&RequestedContext>,
) -> Result<FetchedDetails, String> {
    storefront_engine(retailer)?.details_for_locale(fetcher, token, context, id)
}
