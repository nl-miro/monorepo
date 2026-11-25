//! Sourcing's policy over types received from the retailer libraries.
//!
//! Which locale contexts an initial fill crawls is decided by sourcing, not by
//! `retailer_sauce`; this moved here from `retailer_sauce::request_initial_fill` in 0.3.0.
//! Every initial-fill storefront uses `retailer_sauce`'s shared Shopify engine,
//! built from the retailer's storefront configuration.

use retailer_data_extractor::get_storefront_config;
use retailer_sauce::shopify::graphql::Engine;
use shared::retailer::RetailerCode;

/// The Shopify engine for a retailer's storefront, or an error when the
/// retailer has no storefront configuration.
pub(crate) fn storefront_engine(retailer: RetailerCode) -> Result<Engine, String> {
    get_storefront_config(retailer)
        .map(Engine)
        .ok_or_else(|| "initial fill is not configured".into())
}

#[cfg(test)]
mod tests {
    use retailer_data_extractor::get_storefront_config;
    use retailer_sauce::prelude::{
        Language, Locale, Market, Region, RetailerCode, RetailerProfile, StorefrontAccess,
        StorefrontConfig, StorefrontStrategy, SupportedLocales,
    };

    #[test]
    fn inventory_contexts_exclude_advertised_entries_and_preserve_countryless_languages() {
        let contexts = |code| {
            get_storefront_config(code)
                .unwrap()
                .initial_fill_contexts()
                .unwrap()
        };
        let ca = contexts(RetailerCode::AnkerCa);
        assert_eq!(
            ca.iter()
                .map(|c| (c.country.as_deref(), c.language.as_str()))
                .collect::<Vec<_>>(),
            [(Some("CA"), "EN"), (Some("CA"), "FR")]
        );
        assert_eq!(
            contexts(RetailerCode::AnkerEu)[0].country.as_deref(),
            Some("AT")
        );
        assert_eq!(contexts(RetailerCode::AnkerDe).len(), 1);
        let eu = contexts(RetailerCode::MinisForumEu);
        assert_eq!(
            eu.iter()
                .map(|c| (c.country.as_deref(), c.language.as_str()))
                .collect::<Vec<_>>(),
            [(None, "EN")]
        );
        let main = contexts(RetailerCode::UgreenCom);
        assert_eq!(main.len(), 2);
        assert!(main.iter().all(|c| c.currency.is_none()));
    }

    #[test]
    fn empty_locales_and_regional_countries_are_rejected() {
        const REGION_LOCALE: [SupportedLocales; 1] = [SupportedLocales::new(
            Locale::en_EU,
            "https://example.com",
            Some(Market::Region(Region::EuropeanUnion)),
            Language::EN,
        )];
        let config = |supported_locales| {
            StorefrontConfig::new(
                RetailerProfile {
                    retailer_code: RetailerCode::AnkerAu,
                    storefront_website: "https://example.com",
                    target_markets: &[],
                    shipping_countries: &[],
                    supported_locales,
                },
                StorefrontAccess {
                    preferred: StorefrontStrategy::ShopifyGraphQl,
                    shopify_shop: None,
                    storefront_graphql_api: None,
                    graphql_api_status: None,
                    acquire_token: false,
                    token_required: false,
                    storefront_url: "https://example.com",
                    graphql_url: "https://example.com/graphql",
                    token_market: None,
                    token_environment: None,
                    default_language: "EN",
                },
            )
        };
        assert!(config(&[]).initial_fill_contexts().is_err());
        assert!(config(&REGION_LOCALE).initial_fill_contexts().is_err());
    }
}
