//! Step that fetches offer details.

use crate::platform::bus::{Command, Event, Handle};
use crate::steps::acquired_at;
use crate::v2::prelude::storefront_engine;
use chrono::DateTime;
use chrono::Utc;
use retailer_sauce::initial_fill::FetchedDetails;
use retailer_sauce::initial_fill::LocaleInitialFillIntegration;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::MarketContext;
use retailer_sauce::prelude::{Http, StorefrontToken};
use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use shared::retailer::RetailerCode;
use std::collections::HashMap;
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
    fn store(&self, body: &str, acquired_at: DateTime<Utc>)
    -> Result<OfferStorageMetadata, String>;
}
impl Store for ObjectStorage {
    fn store(
        &self,
        body: &str,
        acquired_at: DateTime<Utc>,
    ) -> Result<OfferStorageMetadata, String> {
        self.create_offer(BUCKET, body, acquired_at)
            .map_err(|e| e.to_string())
    }
}
pub struct Handler<F, S> {
    pub fetcher: F,
    pub store: S,
    pub clock: fn() -> DateTime<Utc>,
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
        let acquired_at = acquired_at((self.clock)());
        let body = stamp_acquired_at(&details.body, acquired_at)?;
        let storage = self.store.store(&body, acquired_at)?;
        info!(context = ?command.context, product_id, object_storage_id = %storage.object_storage_id, "complete offer details stored");
        let fetched = OfferDetailsFetched {
            context: details.context,
            retailer_code: command.retailer_code,
            source: command.source,
            acquired_at,
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
    pub acquired_at: DateTime<Utc>,
    pub storage: OfferStorageMetadata,
}

impl Event for OfferDetailsFetched {
    const NAME: &'static str = "OfferDetailsFetched";
}

/// Records `acquired_at` at the start of the raw root object, keeping every other byte.
pub(crate) fn stamp_acquired_at(body: &str, acquired_at: DateTime<Utc>) -> Result<String, String> {
    if body.starts_with('\u{feff}') {
        return Err("raw offer must not start with a byte order mark".into());
    }
    // Ignoring values validates the whole document without converting its numbers.
    let root: HashMap<String, IgnoredAny> =
        from_str(body).map_err(|e| format!("raw offer must be a JSON object: {e}"))?;
    if root.contains_key("acquired_at") {
        return Err("raw offer already contains acquired_at".into());
    }
    // The document is an object, so its first byte after JSON whitespace is the brace.
    let brace = body.len() - body.trim_start_matches([' ', '\t', '\r', '\n']).len();
    let timestamp = to_string(&acquired_at).map_err(|e| e.to_string())?;
    let separator = if root.is_empty() { "" } else { "," };
    Ok(format!(
        "{}{{\"acquired_at\":{timestamp}{separator}{}",
        &body[..brace],
        &body[brace + 1..]
    ))
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

#[cfg(test)]
mod stamp_tests {
    use super::*;

    fn time() -> DateTime<Utc> {
        acquired_at("2026-11-12T09:10:11.123456789Z".parse().unwrap())
    }

    const STAMP: &str = r#""acquired_at":"2026-11-12T09:10:11.123456Z""#;

    #[test]
    fn capture_truncates_to_microseconds() {
        assert_eq!(
            time(),
            "2026-11-12T09:10:11.123456Z"
                .parse::<DateTime<Utc>>()
                .unwrap()
        );
    }

    #[test]
    fn stamp_preserves_every_other_byte() {
        // A long numeric literal, unsorted keys and escapes must survive verbatim.
        let body = " \t\r\n{\"z\":12345678901234567890.12345678901234567890e-3,\"a\":\"\\u00e9\\\"q\\\\\",\"m\":[1.0, 2e400]}\n";
        let stamped = stamp_acquired_at(body, time()).unwrap();
        assert_eq!(
            stamped,
            format!(
                " \t\r\n{{{STAMP},\"z\":12345678901234567890.12345678901234567890e-3,\"a\":\"\\u00e9\\\"q\\\\\",\"m\":[1.0, 2e400]}}\n"
            )
        );
        assert_eq!(stamped.replacen(&format!("{STAMP},"), "", 1), body);
    }

    #[test]
    fn stamp_omits_the_separator_for_an_empty_object() {
        for (body, expected) in [
            ("{}", format!("{{{STAMP}}}")),
            ("{ }", format!("{{{STAMP} }}")),
            ("\n{\n}\n", format!("\n{{{STAMP}\n}}\n")),
        ] {
            assert_eq!(stamp_acquired_at(body, time()).unwrap(), expected);
        }
    }

    #[test]
    fn stamp_rejects_a_bom_a_non_object_or_an_existing_key() {
        for body in [
            "\u{feff}{}",
            "[]",
            "null",
            "\"text\"",
            "{",
            "{} {}",
            "{\"acquired_at\":null}",
            "{\"a\":1,\"acquired\\u005fat\":\"2026-11-12T09:10:11Z\"}",
        ] {
            assert!(stamp_acquired_at(body, time()).is_err(), "{body:?}");
        }
    }
}
