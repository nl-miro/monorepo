//! Step that acquires a storefront token.

pub(crate) mod prelude;

mod command {
    use crate::platform::bus::Command;
    use serde::{Deserialize, Serialize};
    use shared::retailer::RetailerCode;

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
}

mod event {
    use crate::platform::bus::Event;
    use chrono::{DateTime, Utc};
    use retailer_sauce::prelude::StorefrontToken;
    use serde::{Deserialize, Serialize};
    use shared::retailer::RetailerCode;

    /// Resolves storefront access. None means an optional public token was not found.
    /// Historical events containing a token string still deserialize as Some.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct StorefrontTokenAcquired {
        pub storefront_token: Option<StorefrontToken>,
        pub retailer_code: RetailerCode,
        pub url: String,
        pub acquired_at: DateTime<Utc>,
    }

    impl Event for StorefrontTokenAcquired {
        const NAME: &'static str = "StorefrontTokenAcquired";
    }
}

pub use self::command::AcquireStorefrontToken;
pub use self::event::StorefrontTokenAcquired;

use super::storefront_engine;
use crate::platform::bus::Handle;
use chrono::Utc;
pub use retailer_sauce::prelude::Http as Fetch;
use retailer_sauce::prelude::InitialFillIntegration;
use tracing::info;

pub struct Handler<F>(pub F);
impl<F: Fetch + Send + Sync + 'static> Handle for Handler<F> {
    type Command = AcquireStorefrontToken;
    type Event = StorefrontTokenAcquired;
    type Error = String;
    fn handle(&self, command: AcquireStorefrontToken) -> Result<Vec<Self::Event>, String> {
        let engine = storefront_engine(command.retailer_code)?;

        let storefront_token = engine.resolve_access(&self.0, &command.url)?;

        info!(retailer = ?command.retailer_code, has_token = storefront_token.is_some(), "Storefront access resolved");

        let acquired = StorefrontTokenAcquired {
            retailer_code: command.retailer_code,
            url: command.url,
            storefront_token,
            acquired_at: Utc::now(),
        };

        Ok(vec![acquired])
    }
}
