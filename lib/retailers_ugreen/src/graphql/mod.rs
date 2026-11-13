pub(crate) mod operations;
pub mod raw;
use retailer_sauce::shopify::markets::MarketContext;
use retailers_wip::initial_fill::{
    FetchedDetails, LocaleCatalogBatch, LocaleInitialFillIntegration, RequestedContext,
};
/// Continuations identify the owning product/variant, never a translated label.
#[derive(Debug, Clone)]
pub enum Operation {
    Markets,
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
    SellingPlans {
        id: String,
        group_after: Option<String>,
        group_cursor: String,
        after: Option<String>,
    },
}

mod artifact;
mod auth;
mod catalog;
mod fetch;
pub mod markets;
mod scripts;
mod token;
use retailer_sauce::initial_fill::{CatalogBatch, Http, InitialFillIntegration};
use retailer_sauce::shopify::initial_fill::{GraphqlRequest, StorefrontConfig, StorefrontToken};
use serde_json::Value;
pub struct Engine {
    pub code: &'static str,
    pub config: StorefrontConfig,
    pub all_markets: bool,
}
impl InitialFillIntegration for Engine {
    fn config(&self) -> StorefrontConfig {
        self.config
    }
    fn all_market_contexts(&self) -> bool {
        self.all_markets
    }
    fn resolve_access(
        &self,
        http: &dyn Http,
        url: &str,
    ) -> Result<Option<StorefrontToken>, String> {
        auth::resolve(self, http, url)
    }
    fn discover_markets(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
    ) -> Result<Vec<MarketContext>, String> {
        markets::contexts(&self.execute(http, token, &Operation::Markets)?)
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
        let response = http.post(config.graphql_url, &request, token)?;
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
            Operation::Markets => None,
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
            TypeId::of::<crate::prelude::ugreenca_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreencom_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreende_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreeneu_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreenfr_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreenin_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreenjp_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreenkr_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennas_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennasau_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennasca_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennasde_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennases_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennaseu_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennasfr_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennasit_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennasjp_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennasuk_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreennl_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreenuk_raw_v2::RawOfferDetails>()
        );
        assert_eq!(
            TypeId::of::<super::raw::RawOfferDetails>(),
            TypeId::of::<crate::prelude::ugreenus_raw_v2::RawOfferDetails>()
        );
    }
}

#[cfg(test)]
mod optional_country_tests;
