pub mod anker;
pub mod minisforum;
pub mod ugreen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retailer {
    pub retailer_code: String,
    pub storefront_website: String,
    pub shopify_shop: Option<String>,
    pub storefront_graphql_api: Option<String>,
    pub target_markets: Option<Vec<String>>,
    pub target_markets_annotation: Option<String>,
    pub shipping_countries: Option<Vec<String>>,
    pub shipping_restrictions: Option<String>,
    pub graphql_api_status: Option<String>,
    pub supported_locales: Vec<SupportedLocales>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedLocales {
    pub locale: String,
    pub url: String,
    pub country_region: Option<String>,
    pub language: String,
    pub annotation: Option<String>,
}

#[cfg(test)]
mod tests;
