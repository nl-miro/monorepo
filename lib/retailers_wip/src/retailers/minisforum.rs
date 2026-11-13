//! Retailer inventory from `docs/retailer.md`.

use super::{Retailer, SupportedLocales};

/// Returns the documented retailer for an exact, lowercase retailer code.
pub fn get_retailer(retailer_code: &str) -> Option<Retailer> {
    match retailer_code {
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-australia).
        // Keep this record synchronized with its inventory tables.
        "minisforumau" => Some(Retailer {
            retailer_code: "minisforumau".into(),
            storefront_website: "https://au.minisforum.com/".into(),
            shopify_shop: Some("j94nqi-71.myshopify.com".into()),
            storefront_graphql_api: Some(
                "https://j94nqi-71.myshopify.com/api/2026-10/graphql.json".into(),
            ),
            target_markets: Some(vec!["AU".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["AU".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales {
                locale: "en".into(),
                url: "https://au.minisforum.com/".into(),
                country_region: Some("AU".into()),
                language: "EN".into(),
                annotation: None,
            }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-canada).
        // Keep this record synchronized with its inventory tables.
        "minisforumca" => Some(Retailer {
            retailer_code: "minisforumca".into(),
            storefront_website: "https://ca.minisforum.com/".into(),
            shopify_shop: Some("seaminisforum.myshopify.com".into()),
            storefront_graphql_api: Some(
                "https://seaminisforum.myshopify.com/api/2026-10/graphql.json".into(),
            ),
            target_markets: Some(vec!["CA".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["CA".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales {
                locale: "en".into(),
                url: "https://ca.minisforum.com/".into(),
                country_region: Some("CA".into()),
                language: "EN".into(),
                annotation: None,
            }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-hong-kong).
        // Keep this record synchronized with its inventory tables.
        "minisforumhk" => Some(Retailer {
            retailer_code: "minisforumhk".into(),
            storefront_website: "https://minisforum.hk/".into(),
            shopify_shop: Some("minisforumcn.myshopify.com".into()),
            storefront_graphql_api: Some(
                "https://minisforumcn.myshopify.com/api/2026-10/graphql.json".into(),
            ),
            target_markets: Some(vec!["HK".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["HK".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales {
                locale: "zh-TW".into(),
                url: "https://minisforum.hk/".into(),
                country_region: Some("HK".into()),
                language: "ZH_TW".into(),
                annotation: None,
            }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-south-korea).
        // Keep this record synchronized with its inventory tables.
        "minisforumkr" => Some(Retailer {
            retailer_code: "minisforumkr".into(),
            storefront_website: "https://minisforum.kr/".into(),
            shopify_shop: Some("minisforumkr.myshopify.com".into()),
            storefront_graphql_api: Some(
                "https://minisforumkr.myshopify.com/api/2026-10/graphql.json".into(),
            ),
            target_markets: Some(vec!["KR".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["KR".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales {
                locale: "ko".into(),
                url: "https://minisforum.kr/".into(),
                country_region: Some("KR".into()),
                language: "KO".into(),
                annotation: None,
            }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-russia).
        // Keep this record synchronized with its inventory tables.
        "minisforumru" => Some(Retailer {
            retailer_code: "minisforumru".into(),
            storefront_website: "https://minisforum.ru/".into(),
            shopify_shop: Some("store-minisforum-ru.myshopify.com".into()),
            storefront_graphql_api: Some(
                "https://store-minisforum-ru.myshopify.com/api/2026-10/graphql.json".into(),
            ),
            target_markets: Some(vec!["RU".into()]),
            target_markets_annotation: None,
            shipping_countries: None,
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales {
                locale: "ru".into(),
                url: "https://minisforum.ru/".into(),
                country_region: Some("RU".into()),
                language: "RU".into(),
                annotation: None,
            }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-europe).
        // Keep this record synchronized with its inventory tables.
        "minisforumeu" => Some(Retailer {
            retailer_code: "minisforumeu".into(),
            storefront_website: "https://minisforumpc.eu/".into(),
            shopify_shop: Some("minisforum-eu.myshopify.com".into()),
            storefront_graphql_api: Some(
                "https://minisforum-eu.myshopify.com/api/2026-10/graphql.json".into(),
            ),
            target_markets: Some(vec!["EU".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec![
                "CZ".into(),
                "DK".into(),
                "AT".into(),
                "BE".into(),
                "BG".into(),
                "HR".into(),
                "EE".into(),
                "FI".into(),
                "FR".into(),
                "DE".into(),
                "GR".into(),
                "IE".into(),
                "IT".into(),
                "LV".into(),
                "LT".into(),
                "LU".into(),
                "NL".into(),
                "PL".into(),
                "PT".into(),
                "RO".into(),
                "SK".into(),
                "SI".into(),
                "ES".into(),
                "SE".into(),
                "HU".into(),
                "CH".into(),
                "AL".into(),
                "BY".into(),
                "MK".into(),
                "IS".into(),
                "BA".into(),
                "VA".into(),
                "LI".into(),
                "MC".into(),
                "SM".into(),
            ]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![
                SupportedLocales {
                    locale: "en".into(),
                    url: "https://minisforumpc.eu/".into(),
                    country_region: None,
                    language: "EN".into(),
                    annotation: None,
                },
                SupportedLocales {
                    locale: "de".into(),
                    url: "https://minisforumpc.eu/de".into(),
                    country_region: None,
                    language: "DE".into(),
                    annotation: Some("advertised only".into()),
                },
            ],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-france).
        // Keep this record synchronized with its inventory tables.
        "minisforumfr" => Some(Retailer {
            retailer_code: "minisforumfr".into(),
            storefront_website: "https://minisforumpc.fr/".into(),
            shopify_shop: Some("minisforum-fr.myshopify.com".into()),
            storefront_graphql_api: Some(
                "https://minisforum-fr.myshopify.com/api/2026-10/graphql.json".into(),
            ),
            target_markets: Some(vec!["FR".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["BE".into(), "FR".into(), "LU".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![
                SupportedLocales {
                    locale: "fr".into(),
                    url: "https://minisforumpc.fr/".into(),
                    country_region: Some("FR".into()),
                    language: "FR".into(),
                    annotation: None,
                },
                SupportedLocales {
                    locale: "en".into(),
                    url: "https://minisforumpc.fr/en".into(),
                    country_region: None,
                    language: "EN".into(),
                    annotation: Some("advertised only".into()),
                },
            ],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-united-states).
        // Keep this record synchronized with its inventory tables.
        "minisforumus" => Some(Retailer {
            retailer_code: "minisforumus".into(),
            storefront_website: "https://store.minisforum.com/".into(),
            shopify_shop: Some("minisforum.myshopify.com".into()),
            storefront_graphql_api: Some(
                "https://minisforum.myshopify.com/api/2026-10/graphql.json".into(),
            ),
            target_markets: Some(vec!["US".into()]),
            target_markets_annotation: Some("and others (not enumerated)".into()),
            shipping_countries: Some(vec![
                "US".into(),
                "CA".into(),
                "AE".into(),
                "AU".into(),
                "SA".into(),
                "PH".into(),
                "NZ".into(),
                "AG".into(),
                "BS".into(),
                "BB".into(),
                "BZ".into(),
                "BM".into(),
                "KY".into(),
                "DO".into(),
                "EC".into(),
                "SV".into(),
                "GD".into(),
                "HN".into(),
                "JM".into(),
                "LR".into(),
                "MX".into(),
                "NI".into(),
                "PA".into(),
                "KN".into(),
                "LC".into(),
                "TT".into(),
                "TC".into(),
                "VE".into(),
                "FJ".into(),
                "PG".into(),
                "WS".into(),
                "SB".into(),
                "TO".into(),
                "VU".into(),
                "SG".into(),
                "ID".into(),
                "MY".into(),
                "IL".into(),
                "KW".into(),
            ]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![
                SupportedLocales {
                    locale: "en".into(),
                    url: "https://store.minisforum.com/".into(),
                    country_region: Some("US".into()),
                    language: "EN".into(),
                    annotation: None,
                },
                SupportedLocales {
                    locale: "en-CA".into(),
                    url: "https://store.minisforum.com/en-ca".into(),
                    country_region: None,
                    language: "EN".into(),
                    annotation: Some("advertised only".into()),
                },
                SupportedLocales {
                    locale: "en-CL".into(),
                    url: "https://store.minisforum.com/en-os".into(),
                    country_region: None,
                    language: "EN".into(),
                    annotation: Some("advertised only".into()),
                },
                SupportedLocales {
                    locale: "en-PE".into(),
                    url: "https://store.minisforum.com/en-os".into(),
                    country_region: None,
                    language: "EN".into(),
                    annotation: Some("advertised only".into()),
                },
            ],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-japan).
        // Keep this record synchronized with its inventory tables.
        "minisforumjp" => Some(Retailer {
            retailer_code: "minisforumjp".into(),
            storefront_website: "https://www.minisforum.jp/".into(),
            shopify_shop: Some("minisforum-jp.myshopify.com".into()),
            storefront_graphql_api: Some(
                "https://minisforum-jp.myshopify.com/api/2026-10/graphql.json".into(),
            ),
            target_markets: Some(vec!["JP".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["JP".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales {
                locale: "ja".into(),
                url: "https://www.minisforum.jp/".into(),
                country_region: Some("JP".into()),
                language: "JA".into(),
                annotation: None,
            }],
        }),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-united-kingdom).
        // Keep this record synchronized with its inventory tables.
        "minisforumuk" => Some(Retailer {
            retailer_code: "minisforumuk".into(),
            storefront_website: "https://www.minisforum.uk/".into(),
            shopify_shop: Some("minisforum-uk.myshopify.com".into()),
            storefront_graphql_api: Some(
                "https://minisforum-uk.myshopify.com/api/2026-10/graphql.json".into(),
            ),
            target_markets: Some(vec!["GB".into()]),
            target_markets_annotation: None,
            shipping_countries: Some(vec!["IE".into(), "GB".into()]),
            shipping_restrictions: None,
            graphql_api_status: None,
            supported_locales: vec![SupportedLocales {
                locale: "en".into(),
                url: "https://www.minisforum.uk/".into(),
                country_region: Some("GB".into()),
                language: "EN".into(),
                annotation: None,
            }],
        }),
        _ => None,
    }
}
