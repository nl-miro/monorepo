//! Retailer inventory from `docs/retailer.md`.

use super::{Retailer, SupportedLocales};

/// Returns the documented retailer for an exact, lowercase retailer code.
pub fn get_retailer(retailer_code: &str) -> Option<Retailer> {
    match retailer_code {
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-australia).
        // Keep this record synchronized with its inventory tables.
        "ankerau" => Some(Retailer {
            retailer_code: "ankerau".into(),
            storefront_website: "https://www.anker.com/au".into(),
            shopify_shop: Some("oneanker-au.myshopify.com".into()),
            storefront_graphql_api: Some("https://oneanker-au.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["AU".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["AU".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-AU".into(), url: "https://www.anker.com/au".into(), country_region: Some("AU".into()), language: "EN".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-canada).
        // Keep this record synchronized with its inventory tables.
        "ankerca" => Some(Retailer {
            retailer_code: "ankerca".into(),
            storefront_website: "https://www.anker.com/ca".into(),
            shopify_shop: Some("anker-canada.myshopify.com".into()),
            storefront_graphql_api: Some("https://anker-canada.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["CA".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["CA".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-CA".into(), url: "https://www.anker.com/ca".into(), country_region: Some("CA".into()), language: "EN".into(), annotation: None },
SupportedLocales { locale: "fr-CA".into(), url: "https://www.anker.com/ca-fr".into(), country_region: Some("CA".into()), language: "FR".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-germany).
        // Keep this record synchronized with its inventory tables.
        "ankerde" => Some(Retailer {
            retailer_code: "ankerde".into(),
            storefront_website: "https://www.anker.com/de".into(),
            shopify_shop: Some("oneanker-de.myshopify.com".into()),
            storefront_graphql_api: Some("https://oneanker-de.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["DE".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["DE".into()]),
            shipping_restrictions: Some("Excludes standard delivery to postcodes 27498 and 78266; additional express exclusions.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "de".into(), url: "https://www.anker.com/de".into(), country_region: Some("DE".into()), language: "DE".into(), annotation: Some("HTML".into()) },
SupportedLocales { locale: "de-DE".into(), url: "https://www.anker.com/de".into(), country_region: Some("DE".into()), language: "DE".into(), annotation: Some("alternate link".into()) }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-europe).
        // Keep this record synchronized with its inventory tables.
        "ankereu" => Some(Retailer {
            retailer_code: "ankereu".into(),
            storefront_website: "https://www.anker.com/eu-en".into(),
            shopify_shop: Some("ankereu.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankereu.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["EU".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["AT".into(), "BE".into(), "CZ".into(), "HU".into(), "LU".into(), "PL".into(), "PT".into(), "NL".into()]),
            shipping_restrictions: Some("Power banks excluded for Portugal; Solarbank 2 AC limited to Austria. Negative exclusions elsewhere on the page do not add delivery countries.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-EU".into(), url: "https://www.anker.com/eu-en".into(), country_region: Some("AT".into()), language: "EN".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-south-korea).
        // Keep this record synchronized with its inventory tables.
        "ankerkr" => Some(Retailer {
            retailer_code: "ankerkr".into(),
            storefront_website: "https://ankerkorea.co.kr/".into(),
            shopify_shop: None,
            storefront_graphql_api: None,
            target_markets: Some(vec!["KR".into()]),
            target_markets_annotation: None,
            shipping_countries: None,
            shipping_restrictions: None,
            graphql_api_status: Some("Unknown; no endpoint established in the research".into()),
            supported_locales: vec![SupportedLocales { locale: "ko".into(), url: "https://ankerkorea.co.kr/".into(), country_region: Some("KR".into()), language: "KO".into(), annotation: None },
SupportedLocales { locale: "ko-KR".into(), url: "https://ankerkorea.co.kr/".into(), country_region: Some("KR".into()), language: "KO".into(), annotation: Some("alternate link".into()) }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-united-states).
        // Keep this record synchronized with its inventory tables.
        "ankercom" => Some(Retailer {
            retailer_code: "ankercom".into(),
            storefront_website: "https://www.anker.com/".into(),
            shopify_shop: Some("ankerus.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankerus.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["US".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["US".into()]),
            shipping_restrictions: Some("Product/address restrictions apply.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-US".into(), url: "https://www.anker.com/".into(), country_region: Some("US".into()), language: "EN".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-poland).
        // Keep this record synchronized with its inventory tables.
        "ankerpl" => Some(Retailer {
            retailer_code: "ankerpl".into(),
            storefront_website: "https://www.anker.com/eu-pl".into(),
            shopify_shop: Some("ankereu.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankereu.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["PL".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["AT".into(), "BE".into(), "BG".into(), "HR".into(), "CZ".into(), "DK".into(), "EE".into(), "FI".into(), "FR".into(), "GR".into(), "ES".into(), "NL".into(), "LT".into(), "LU".into(), "DE".into(), "PL".into(), "PT".into(), "RO".into(), "SE".into(), "SK".into(), "SI".into(), "HU".into(), "IT".into(), "LV".into()]),
            shipping_restrictions: Some("Destinations explicitly listed in Polish; not inferred from the eu-pl code.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "pl-PL".into(), url: "https://www.anker.com/eu-pl".into(), country_region: Some("PL".into()), language: "PL".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-france).
        // Keep this record synchronized with its inventory tables.
        "ankerfr" => Some(Retailer {
            retailer_code: "ankerfr".into(),
            storefront_website: "https://www.anker.com/fr".into(),
            shopify_shop: Some("ankerfra.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankerfra.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["FR".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["FR".into(), "IT".into(), "ES".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "fr-FR".into(), url: "https://www.anker.com/fr".into(), country_region: Some("FR".into()), language: "FR".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-new-zealand).
        // Keep this record synchronized with its inventory tables.
        "ankernz" => Some(Retailer {
            retailer_code: "ankernz".into(),
            storefront_website: "https://www.anker.com/nz".into(),
            shopify_shop: Some("ankernz.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankernz.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: None,
            target_markets_annotation: Some("conflicting evidence".into()),
            shipping_countries: None,
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-NZ".into(), url: "https://www.anker.com/nz".into(), country_region: Some("NZ".into()), language: "EN".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-united-kingdom).
        // Keep this record synchronized with its inventory tables.
        "ankeruk" => Some(Retailer {
            retailer_code: "ankeruk".into(),
            storefront_website: "https://www.anker.com/uk".into(),
            shopify_shop: Some("oneanker-uk.myshopify.com".into()),
            storefront_graphql_api: Some("https://oneanker-uk.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["GB".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["GB".into()]),
            shipping_restrictions: Some("Some postcodes and PO boxes excluded; express coverage is narrower.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-GB".into(), url: "https://www.anker.com/uk".into(), country_region: Some("GB".into()), language: "EN".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-vietnam).
        // Keep this record synchronized with its inventory tables.
        "ankervn" => Some(Retailer {
            retailer_code: "ankervn".into(),
            storefront_website: "https://www.anker.com/vn".into(),
            shopify_shop: Some("ankervnm.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankervnm.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: None,
            target_markets_annotation: None,
            shipping_countries: None,
            shipping_restrictions: None,
            graphql_api_status: Some("Responded with GraphQL error: Online Store channel is locked.".into()),
            supported_locales: vec![SupportedLocales { locale: "vi-VN".into(), url: "https://www.anker.com/vn".into(), country_region: Some("VN".into()), language: "VI".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-italy).
        // Keep this record synchronized with its inventory tables.
        "ankeritalycom" => Some(Retailer {
            retailer_code: "ankeritalycom".into(),
            storefront_website: "https://www.ankeritaly.com/".into(),
            shopify_shop: Some("anker-italy.myshopify.com".into()),
            storefront_graphql_api: Some("https://anker-italy.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["IT".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["IT".into(), "FR".into(), "ES".into(), "DE".into(), "MT".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "it-IT".into(), url: "https://www.ankeritaly.com/".into(), country_region: Some("IT".into()), language: "IT".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-japan).
        // Keep this record synchronized with its inventory tables.
        "ankerjapancom" => Some(Retailer {
            retailer_code: "ankerjapancom".into(),
            storefront_website: "https://www.ankerjapan.com/".into(),
            shopify_shop: Some("anker-japan-official-site.myshopify.com".into()),
            storefront_graphql_api: Some("https://anker-japan-official-site.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["JP".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["JP".into()]),
            shipping_restrictions: Some("Some Okinawa addresses excluded; forwarding services not accepted.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "ja".into(), url: "https://www.ankerjapan.com/".into(), country_region: Some("JP".into()), language: "JA".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-nordics).
        // Keep this record synchronized with its inventory tables.
        // Historical combined identity; new work uses the five country codes below.
        "ankernordicscom" => Some(Retailer {
            retailer_code: "ankernordicscom".into(),
            storefront_website: "https://www.ankernordics.com/".into(),
            shopify_shop: Some("ankerneu.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["DK".into(), "FI".into(), "IS".into(), "NO".into(), "SE".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["SE".into(), "NO".into(), "DK".into(), "FI".into(), "IS".into()]),
            shipping_restrictions: Some("Iceland import clearance and product-specific exclusions apply.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-SE".into(), url: "https://www.ankernordics.com/".into(), country_region: Some("SE".into()), language: "EN".into(), annotation: None },
SupportedLocales { locale: "en-dk".into(), url: "https://www.ankernordics.com/".into(), country_region: Some("DK".into()), language: "EN".into(), annotation: Some("alternate link".into()) },
SupportedLocales { locale: "en-fi".into(), url: "https://www.ankernordics.com/".into(), country_region: Some("FI".into()), language: "EN".into(), annotation: Some("alternate link".into()) },
SupportedLocales { locale: "en-is".into(), url: "https://www.ankernordics.com/".into(), country_region: Some("IS".into()), language: "EN".into(), annotation: Some("alternate link".into()) },
SupportedLocales { locale: "en-no".into(), url: "https://www.ankernordics.com/".into(), country_region: Some("NO".into()), language: "EN".into(), annotation: Some("alternate link".into()) }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-sweden).
        // Keep this record synchronized with its inventory tables.
        "ankerse" => Some(Retailer {
            retailer_code: "ankerse".into(),
            storefront_website: "https://www.ankernordics.com/".into(),
            shopify_shop: Some("ankerneu.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["SE".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["SE".into(), "NO".into(), "DK".into(), "FI".into(), "IS".into()]),
            shipping_restrictions: Some("Iceland import clearance and product-specific exclusions apply.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-SE".into(), url: "https://www.ankernordics.com/".into(), country_region: Some("SE".into()), language: "EN".into(), annotation: None }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-norway).
        // Keep this record synchronized with its inventory tables.
        "ankerno" => Some(Retailer {
            retailer_code: "ankerno".into(),
            storefront_website: "https://www.ankernordics.com/".into(),
            shopify_shop: Some("ankerneu.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["NO".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["SE".into(), "NO".into(), "DK".into(), "FI".into(), "IS".into()]),
            shipping_restrictions: Some("Iceland import clearance and product-specific exclusions apply.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-no".into(), url: "https://www.ankernordics.com/".into(), country_region: Some("NO".into()), language: "EN".into(), annotation: Some("alternate link".into()) }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-finland).
        // Keep this record synchronized with its inventory tables.
        "ankerfi" => Some(Retailer {
            retailer_code: "ankerfi".into(),
            storefront_website: "https://www.ankernordics.com/".into(),
            shopify_shop: Some("ankerneu.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["FI".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["SE".into(), "NO".into(), "DK".into(), "FI".into(), "IS".into()]),
            shipping_restrictions: Some("Iceland import clearance and product-specific exclusions apply.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-fi".into(), url: "https://www.ankernordics.com/".into(), country_region: Some("FI".into()), language: "EN".into(), annotation: Some("alternate link".into()) }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-iceland).
        // Keep this record synchronized with its inventory tables.
        "ankeris" => Some(Retailer {
            retailer_code: "ankeris".into(),
            storefront_website: "https://www.ankernordics.com/".into(),
            shopify_shop: Some("ankerneu.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["IS".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["SE".into(), "NO".into(), "DK".into(), "FI".into(), "IS".into()]),
            shipping_restrictions: Some("Iceland import clearance and product-specific exclusions apply.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-is".into(), url: "https://www.ankernordics.com/".into(), country_region: Some("IS".into()), language: "EN".into(), annotation: Some("alternate link".into()) }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-denmark).
        // Keep this record synchronized with its inventory tables.
        "ankerdk" => Some(Retailer {
            retailer_code: "ankerdk".into(),
            storefront_website: "https://www.ankernordics.com/".into(),
            shopify_shop: Some("ankerneu.myshopify.com".into()),
            storefront_graphql_api: Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            target_markets: Some(vec!["DK".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["SE".into(), "NO".into(), "DK".into(), "FI".into(), "IS".into()]),
            shipping_restrictions: Some("Iceland import clearance and product-specific exclusions apply.".into()),
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales { locale: "en-dk".into(), url: "https://www.ankernordics.com/".into(), country_region: Some("DK".into()), language: "EN".into(), annotation: Some("alternate link".into()) }],
        }),
        _ => None,
    }
}
