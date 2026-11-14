pub(crate) mod operations;
pub mod raw;
use retailer_sauce::shopify::markets::MarketContext;
use retailers_wip::initial_fill::{
    FetchedDetails, LocaleCatalogBatch, LocaleInitialFillIntegration, RequestedContext,
};
/// Continuations identify the owning product/variant, never a translated label.
#[derive(Debug, Clone)]
pub enum Operation {
    InContext {
        context: MarketContext,
        operation: Box<Operation>,
    },
    InLocale {
        context: RequestedContext,
        operation: Box<Operation>,
    },
    Catalog {
        first: u16,
        after: Option<String>,
    },
    Details {
        id: String,
    },
    ProductConnection {
        id: String,
        field: String,
        after: Option<String>,
    },
    VariantConnection {
        id: String,
        field: String,
        after: Option<String>,
    },
}

mod artifact;
mod auth;
mod catalog;
mod fetch;
mod scripts;
mod token;
use retailer_sauce::initial_fill::{CatalogBatch, Http, InitialFillIntegration};
use retailer_sauce::shopify::initial_fill::{GraphqlRequest, StorefrontConfig, StorefrontToken};
use serde_json::Value;
pub struct Engine {
    pub code: &'static str,
    pub config: StorefrontConfig,
}
impl InitialFillIntegration for Engine {
    fn config(&self) -> StorefrontConfig {
        self.config
    }
    // Required by retailer_sauce 0.1.1; contexts come from Retailer.supported_locales.
    fn all_market_contexts(&self) -> bool {
        true
    }
    fn resolve_access(
        &self,
        http: &dyn Http,
        url: &str,
    ) -> Result<Option<StorefrontToken>, String> {
        auth::resolve(self, http, url)
    }
    // Required by the external compatibility trait; initial fill uses inventory locales.
    fn discover_markets(
        &self,
        _http: &dyn Http,
        _token: Option<&StorefrontToken>,
    ) -> Result<Vec<MarketContext>, String> {
        Err("remote market discovery was removed; use Retailer.supported_locales".into())
    }
    fn catalog(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        context: Option<&MarketContext>,
        after: Option<String>,
    ) -> Result<CatalogBatch, String> {
        catalog::fetch(
            self,
            http,
            token,
            context.map(RequestedContext::from).as_ref(),
            after,
        )
        .map(|result| result.batch)
    }
    fn details(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        context: Option<&MarketContext>,
        id: &str,
    ) -> Result<String, String> {
        serde_json::to_string(&fetch::fetch_in_context(
            self,
            http,
            token,
            id,
            context.map(RequestedContext::from).as_ref(),
        )?)
        .map_err(|e| e.to_string())
    }
}
impl LocaleInitialFillIntegration for Engine {
    fn catalog_for_locale(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        context: Option<&RequestedContext>,
        after: Option<String>,
    ) -> Result<LocaleCatalogBatch, String> {
        catalog::fetch(self, http, token, context, after)
    }
    fn details_for_locale(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        context: Option<&RequestedContext>,
        id: &str,
    ) -> Result<FetchedDetails, String> {
        let artifact = fetch::fetch_in_context(self, http, token, id, context)?;
        Ok(FetchedDetails {
            body: serde_json::to_string(&artifact).map_err(|e| e.to_string())?,
            context: artifact.context,
        })
    }
}
impl Engine {
    pub fn request(&self, operation: &Operation) -> Result<GraphqlRequest, String> {
        operations::request(operation, &self.config)
    }
    pub fn validate_response(&self, operation: &Operation, response: &Value) -> Result<(), String> {
        operations::validate_response(operation, response, &self.config)
    }
    fn execute(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        operation: &Operation,
    ) -> Result<Value, String> {
        let config = self.config;
        if config.requires_token() && token.is_none() {
            return Err("Storefront authentication does not match retailer configuration".into());
        }
        let request = self.request(operation)?;
        // Ignore historical queued tokens for storefronts configured for anonymous access.
        let request_token = token.filter(|_| config.attempts_token() || config.requires_token());
        let response = http.post(config.graphql_url, &request, request_token)?;
        self.validate_response(operation, &response)
            .map_err(|error| {
                // Redact the active credential if Shopify echoes it in diagnostics.
                match token {
                    Some(token) => error.replace(token.as_str(), "[redacted]"),
                    None => error,
                }
            })?;
        if let Operation::InLocale { context, .. } = operation {
            context.clone().observe(&response)?;
            return Ok(response);
        }
        let expected = match operation {
            Operation::InContext { context, .. } => {
                Some((Some(context.country.as_str()), context.language.as_str()))
            }
            _ => Some((None, config.language)),
        };
        if matches!(operation, Operation::InContext { .. })
            && response.pointer("/extensions/context").is_none()
        {
            return Err("Storefront omitted requested context evidence".into());
        }
        if let Operation::InContext { context, .. } = operation
            && (response
                .pointer("/extensions/context/country")
                .and_then(Value::as_str)
                != Some(context.country.as_str())
                || response
                    .pointer("/extensions/context/language")
                    .and_then(Value::as_str)
                    != Some(context.language.as_str()))
        {
            return Err("Storefront response omitted or changed the requested context".into());
        }
        if let Some((country, language)) = expected
            && let Some(context) = response.pointer("/extensions/context")
            && (country
                .is_some_and(|expected| context.get("country").is_some_and(|c| c != expected))
                || context.get("language").is_some_and(|l| l != language))
        {
            return Err("Storefront response context does not match retailer market".into());
        }
        Ok(response)
    }
}
pub fn product_gid(id: &str) -> Result<(), String> {
    let digits = id
        .strip_prefix("gid://shopify/Product/")
        .ok_or("invalid Shopify product GID")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid Shopify product GID".into());
    }
    Ok(())
}

pub fn next_cursor(connection: &Value) -> Result<Option<String>, String> {
    let more = connection
        .pointer("/pageInfo/hasNextPage")
        .and_then(Value::as_bool)
        .ok_or("missing connection pageInfo")?;
    if !more {
        return Ok(None);
    }
    connection
        .pointer("/pageInfo/endCursor")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(|s| Some(s.to_owned()))
        .ok_or_else(|| "missing continuation cursor".into())
}

#[cfg(test)]
mod sharing_tests {
    use std::any::TypeId;
    #[test]
    fn storefronts_share_brand_raw_model() {
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::minisforumau_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::minisforumca_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::minisforumeu_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::minisforumfr_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::minisforumhk_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::minisforumjp_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::minisforumkr_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::minisforumru_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::minisforumuk_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::minisforumus_raw_v2::RawOfferDetails>()
        );
    }
}

#[cfg(test)]
mod optional_country_tests;
