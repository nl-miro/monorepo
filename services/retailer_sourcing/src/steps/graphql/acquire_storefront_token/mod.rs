//! Step that acquires a storefront token.

use crate::platform::bus::{Command, Event, Handle};
use crate::v2::prelude::storefront_engine;
use chrono::{DateTime, Utc};
pub use retailer_sauce::prelude::Http as Fetch;
use retailer_sauce::prelude::{InitialFillIntegration, StorefrontToken};
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use tracing::info;

/// Acquires a storefront token from the given retailer URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquireStorefrontToken {
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl AcquireStorefrontToken {
    pub fn new(retailer_code: RetailerCode, url: String) -> Self {
        Self { retailer_code, url }
    }
}

impl Command for AcquireStorefrontToken {
    const NAME: &'static str = "AcquireStorefrontToken";
}

pub struct Handler<F> {
    pub fetcher: F,
}

impl<F: Fetch + Send + Sync + 'static> Handle for Handler<F> {
    type Command = AcquireStorefrontToken;
    type Event = StorefrontTokenAcquired;
    type Error = String;
    fn handle(&self, cmd: AcquireStorefrontToken) -> Result<Vec<Self::Event>, String> {
        let rc = cmd.retailer_code;
        let url = cmd.url.clone();

        // TODO: convert implementation to intention
        let engine = storefront_engine(rc)?;
        let token = engine.resolve_access(&self.fetcher, &url)?;

        // TODO: convert implementation to intention
        info!(retailer = ?rc, has_token = token.is_some(), "Storefront access resolved");

        // TODO: convert implementation to intention
        let evt = StorefrontTokenAcquired::new(token, rc, url, Utc::now());

        Ok(vec![evt])
    }
}

/// Resolves storefront access. None means an optional public token was not found.
/// Historical events containing a token string still deserialize as Some.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorefrontTokenAcquired {
    pub storefront_token: Option<StorefrontToken>,
    pub retailer_code: RetailerCode,
    pub url: String,
    pub acquired_at: DateTime<Utc>,
}

impl StorefrontTokenAcquired {
    fn new(
        storefront_token: Option<StorefrontToken>,
        retailer_code: RetailerCode,
        url: String,
        acquired_at: DateTime<Utc>,
    ) -> Self {
        Self {
            storefront_token,
            retailer_code,
            url,
            acquired_at,
        }
    }
}

impl Event for StorefrontTokenAcquired {
    const NAME: &'static str = "StorefrontTokenAcquired";
}
