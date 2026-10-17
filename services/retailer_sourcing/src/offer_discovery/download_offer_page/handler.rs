use uuid::Uuid;

use super::command::DownloadOfferPage;
use super::event::OfferPageWasDownloaded;
use super::store::StoreError;
use crate::platform::bus::Handle;
use crate::platform::http::{self, FetchError};

/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    /// Stores the page, replacing any page stored for the offer before.
    fn store_raw_offer(&self, offer_id: Uuid, url: &str, body: &str) -> Result<(), StoreError>;
}

/// Fetches the body of one offer page.
pub trait FetchOfferPage: Send + Sync + 'static {
    fn fetch(&self, url: &str) -> Result<String, FetchError>;
}

impl FetchOfferPage for http::Client {
    fn fetch(&self, url: &str) -> Result<String, FetchError> {
        self.get(url)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadOfferPageError {
    #[error("offer page download failed for {url}: {error}")]
    Fetch { url: String, error: FetchError },

    #[error(transparent)]
    Store(#[from] StoreError),
}

pub struct Handler<S, F> {
    store: S,
    fetcher: F,
}

impl<S, F> Handler<S, F> {
    pub fn new(store: S, fetcher: F) -> Self {
        Self { store, fetcher }
    }
}

impl<S: Store, F: FetchOfferPage> Handle for Handler<S, F> {
    type Command = DownloadOfferPage;
    type Event = OfferPageWasDownloaded;
    type Error = DownloadOfferPageError;

    fn handle(
        &self,
        command: DownloadOfferPage,
    ) -> Result<Vec<OfferPageWasDownloaded>, DownloadOfferPageError> {
        let body =
            self.fetcher
                .fetch(&command.url)
                .map_err(|error| DownloadOfferPageError::Fetch {
                    url: command.url.clone(),
                    error,
                })?;

        self.store
            .store_raw_offer(command.offer_id, &command.url, &body)?;
        tracing::info!(
            offer_id = %command.offer_id,
            url = %command.url,
            bytes = body.len(),
            "offer page downloaded"
        );

        Ok(vec![OfferPageWasDownloaded {
            offer_id: command.offer_id,
            consolidated_sitemap_id: command.consolidated_sitemap_id,
            retailer_code: command.retailer_code,
            url: command.url,
        }])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use bodul_shared::retailer::RetailerCode;

    use super::*;

    const URL: &str = "https://shop.test/products/a";

    #[derive(Default)]
    struct FakeStore {
        log: Mutex<Vec<String>>,
    }

    impl FakeStore {
        fn log(&self, entry: String) -> Result<(), StoreError> {
            self.log.lock().unwrap().push(entry);
            Ok(())
        }
    }

    impl Store for FakeStore {
        fn store_raw_offer(&self, _: Uuid, url: &str, body: &str) -> Result<(), StoreError> {
            self.log(format!("stored {url}: {body}"))
        }
    }

    /// Serves one page, or fails every request.
    struct Page(Option<&'static str>);

    impl FetchOfferPage for Page {
        fn fetch(&self, _: &str) -> Result<String, FetchError> {
            self.0.map(str::to_string).ok_or_else(|| FetchError {
                message: "403 Forbidden".to_string(),
            })
        }
    }

    fn command() -> DownloadOfferPage {
        DownloadOfferPage {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::UgreenCa,
            url: URL.to_string(),
        }
    }

    #[test]
    fn downloads_and_stores_the_page() {
        let handler = Handler::new(FakeStore::default(), Page(Some("<html>offer</html>")));
        let command = command();

        let events = handler.handle(command.clone()).unwrap();

        assert_eq!(
            events,
            vec![OfferPageWasDownloaded {
                offer_id: command.offer_id,
                consolidated_sitemap_id: command.consolidated_sitemap_id,
                retailer_code: RetailerCode::UgreenCa,
                url: URL.to_string(),
            }]
        );
        assert_eq!(
            *handler.store.log.lock().unwrap(),
            vec![format!("stored {URL}: <html>offer</html>")]
        );
    }

    #[test]
    fn downloads_a_url_again_every_time() {
        let handler = Handler::new(FakeStore::default(), Page(Some("<html>offer</html>")));
        for _ in 0..2 {
            assert_eq!(handler.handle(command()).unwrap().len(), 1);
        }
        assert_eq!(handler.store.log.lock().unwrap().len(), 2);
    }

    #[test]
    fn fails_the_command_when_the_download_fails() {
        let handler = Handler::new(FakeStore::default(), Page(None));

        let error = handler.handle(command()).unwrap_err();
        assert!(matches!(&error, DownloadOfferPageError::Fetch { url, .. } if url == URL));
        assert!(handler.store.log.lock().unwrap().is_empty());
    }
}
