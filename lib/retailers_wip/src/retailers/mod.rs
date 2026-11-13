pub mod anker;
pub mod minisforum;
pub mod ugreen;

pub use shared::retailer::RetailerCode;

mod language;
pub use language::Language;

mod locale;
pub use locale::Locale;

mod market;
pub use market::{Country, Market, Region};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retailer {
    pub retailer_code: RetailerCode,
    pub storefront_website: String,
    pub shopify_shop: Option<String>,
    pub storefront_graphql_api: Option<String>,
    pub target_markets: Vec<Market>,
    pub shipping_countries: Vec<Market>,
    pub graphql_api_status: Option<String>,
    pub supported_locales: Vec<SupportedLocales>,
}

impl Retailer {
    /// Creates a retailer with its storefront, markets and supported locales.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        retailer_code: RetailerCode,
        storefront_website: impl Into<String>,
        shopify_shop: Option<String>,
        storefront_graphql_api: Option<String>,
        target_markets: Vec<Market>,
        shipping_countries: Vec<Market>,
        graphql_api_status: Option<String>,
        supported_locales: Vec<SupportedLocales>,
    ) -> Self {
        Self {
            retailer_code,
            storefront_website: storefront_website.into(),
            shopify_shop,
            storefront_graphql_api,
            target_markets,
            shipping_countries,
            graphql_api_status,
            supported_locales,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedLocales {
    pub locale: Locale,
    pub url: String,
    pub country_region: Option<Market>,
    pub language: Language,
}

impl SupportedLocales {
    /// Creates a supported locale with its URL, market context and language.
    pub fn new(
        locale: Locale,
        url: impl Into<String>,
        country_region: Option<Market>,
        language: Language,
    ) -> Self {
        Self {
            locale,
            url: url.into(),
            country_region,
            language,
        }
    }
}

#[cfg(test)]
mod tests;

/// Loads the inventory entity for a known retailer code.
pub fn get_retailer(code: RetailerCode) -> Option<Retailer> {
    let code = code.as_str();
    anker::get_retailer(&code)
        .or_else(|| minisforum::get_retailer(&code))
        .or_else(|| ugreen::get_retailer(&code))
}

impl Retailer {
    /// Main-table locales eligible for collection, including countryless locales.
    pub fn supported_locales(&self) -> &[SupportedLocales] {
        &self.supported_locales
    }
}
