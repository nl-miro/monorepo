use super::*;

type Factory = fn(&str) -> Option<Retailer>;
const FACTORIES: [Factory; 3] = [
    anker::get_retailer,
    minisforum::get_retailer,
    ugreen::get_retailer,
];

#[test]
fn factories_match_documented_inventory() {
    let inventory = include_str!("../../../../docs/retailer.md");
    let mut counts = [0; 3];
    for section in inventory.split("## ").skip(1) {
        let brand = if section.starts_with("Anker ") {
            0
        } else if section.starts_with("Minisforum ") {
            1
        } else {
            2
        };
        let rows: Vec<Vec<&str>> = section
            .lines()
            .filter(|line| line.starts_with('|'))
            .map(|line| line.trim_matches('|').split('|').map(str::trim).collect())
            .collect();
        let field = |name: &str| {
            rows.iter()
                .find(|row| row.len() == 2 && row[0] == name)
                .map(|row| row[1])
        };
        let code = field("Retailer code").unwrap().trim_matches('`');
        let retailer = FACTORIES[brand](code).unwrap();
        counts[brand] += 1;
        for (index, factory) in FACTORIES.iter().enumerate() {
            if index != brand {
                assert!(factory(code).is_none());
            }
        }
        assert_eq!(retailer.retailer_code, code);
        assert_eq!(
            retailer.storefront_website,
            field("Storefront website").unwrap()
        );
        assert_eq!(
            retailer.shopify_shop.as_deref(),
            field("Shopify shop value").filter(|v| *v != "Unknown")
        );
        assert_eq!(
            retailer.storefront_graphql_api.as_deref(),
            field("Storefront GraphQL API").filter(|v| *v != "Unknown")
        );
        assert_eq!(
            retailer.shipping_restrictions.as_deref(),
            field("Shipping restrictions")
        );
        assert_eq!(
            retailer.graphql_api_status.as_deref(),
            field("GraphQL API status")
        );
        let shipping = field("Shipping countries").unwrap();
        assert_eq!(
            retailer.shipping_countries,
            (shipping != "Unknown").then(|| shipping.split(", ").map(String::from).collect())
        );
        let markets = field("Target markets").unwrap();
        let codes: Vec<String> = markets
            .split('(')
            .skip(1)
            .filter_map(|part| {
                let code = part.split(')').next().unwrap();
                (code.len() == 2 && code.bytes().all(|c| c.is_ascii_uppercase()))
                    .then(|| code.to_owned())
            })
            .collect();
        assert_eq!(
            retailer.target_markets,
            (!codes.is_empty()).then_some(codes)
        );
        let annotation = if markets.contains("conflicting evidence") {
            Some("conflicting evidence")
        } else if markets.contains("not enumerated") {
            Some("and others (not enumerated)")
        } else {
            None
        };
        assert_eq!(retailer.target_markets_annotation.as_deref(), annotation);
        let expected: Vec<SupportedLocales> = rows
            .iter()
            .filter(|row| {
                row.len() == 4 && row[0] != "Supported locales" && !row[0].starts_with('-')
            })
            .flat_map(|row| {
                row[0].split(", ").map(|label| {
                    let (locale, annotation) =
                        label.split_once(" (").map_or((label, None), |(tag, note)| {
                            (tag, Some(note.trim_end_matches(')')))
                        });
                    SupportedLocales {
                        locale: locale.into(),
                        url: row[1].into(),
                        country_region: (row[2] != "Not selected").then(|| row[2].into()),
                        language: row[3].into(),
                        annotation: annotation.map(String::from),
                    }
                })
            })
            .collect();
        assert_eq!(retailer.supported_locales, expected, "{code}");
    }
    assert_eq!(counts, [19, 10, 21]);
}

#[test]
fn invalid_codes_are_not_normalized() {
    for factory in FACTORIES {
        for code in [
            "",
            "unknown",
            "ANKERAU",
            " ankerau",
            "ankerau ",
            "UGREENNAS",
            "MINISFORUMAU",
        ] {
            assert!(factory(code).is_none());
        }
    }
}

#[test]
fn collection_context_is_independent_of_market_and_locale() {
    let eu = anker::get_retailer("ankereu").unwrap();
    assert_eq!(eu.target_markets.unwrap(), ["EU"]);
    assert_eq!(eu.supported_locales[0].locale, "en-EU");
    assert_eq!(
        eu.supported_locales[0].country_region.as_deref(),
        Some("AT")
    );
    for retailer in [
        minisforum::get_retailer("minisforumeu").unwrap(),
        ugreen::get_retailer("ugreennaseu").unwrap(),
    ] {
        assert!(
            retailer
                .supported_locales
                .iter()
                .all(|locale| locale.country_region.is_none())
        );
    }
    let germany = anker::get_retailer("ankerde").unwrap();
    assert_eq!(germany.supported_locales.len(), 2);
    assert_eq!(germany.supported_locales[1].locale, "de-DE");
    assert!(
        anker::get_retailer("ankerkr")
            .unwrap()
            .shopify_shop
            .is_none()
    );
    let main = ugreen::get_retailer("ugreencom").unwrap();
    assert!(
        main.supported_locales
            .iter()
            .filter(|locale| locale.locale == "en")
            .count()
            > 1
    );
    assert!(
        main.supported_locales
            .iter()
            .any(
                |locale| locale.annotation.as_deref() == Some("advertised only")
                    && locale.country_region.is_none()
            )
    );
}

#[test]
fn nordic_markets_keep_distinct_contexts_and_shared_storefront() {
    let legacy = anker::get_retailer("ankernordicscom").unwrap();
    for (code, country) in [
        ("ankerse", "SE"),
        ("ankerno", "NO"),
        ("ankerfi", "FI"),
        ("ankeris", "IS"),
        ("ankerdk", "DK"),
    ] {
        let retailer = anker::get_retailer(code).unwrap();
        assert_eq!(retailer.target_markets, Some(vec![country.into()]));
        assert_eq!(retailer.storefront_website, legacy.storefront_website);
        assert_eq!(retailer.shopify_shop, legacy.shopify_shop);
        assert_eq!(
            retailer.storefront_graphql_api,
            legacy.storefront_graphql_api
        );
        assert_eq!(retailer.shipping_countries, legacy.shipping_countries);
        assert_eq!(retailer.supported_locales.len(), 1);
        assert_eq!(
            retailer.supported_locales[0].country_region.as_deref(),
            Some(country)
        );
        assert_eq!(retailer.supported_locales[0].language, "EN");
    }
}
