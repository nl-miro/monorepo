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
        assert_eq!(
            retailer.retailer_code,
            RetailerCode::from_str(code).unwrap()
        );
        assert_eq!(retailer.retailer_code.as_str(), code);
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
            retailer.graphql_api_status.as_deref(),
            field("GraphQL API status")
        );
        let shipping = field("Shipping countries").unwrap();
        assert_eq!(
            retailer.shipping_countries,
            if shipping == "Unknown" {
                Vec::<Market>::new()
            } else {
                shipping
                    .split(", ")
                    .map(|code| Market::from_code(code).expect("documented shipping country"))
                    .collect::<Vec<_>>()
            }
        );
        let markets = field("Target markets").unwrap();
        let codes: Vec<Market> = markets
            .split('(')
            .skip(1)
            .filter_map(|part| {
                let code = part.split(')').next().unwrap();
                (code.len() == 2 && code.bytes().all(|c| c.is_ascii_uppercase()))
                    .then(|| Market::from_code(code).expect("documented target market"))
            })
            .collect();
        assert_eq!(retailer.target_markets, codes);
        let main_rows: Vec<_> = rows
            .iter()
            .take_while(|row| row[0] != "Advertised-only locales")
            .cloned()
            .collect();
        let mut expected: Vec<SupportedLocales> = main_rows
            .iter()
            .filter(|row| {
                row.len() == 4
                    && !matches!(row[0], "Supported locales" | "Advertised-only locales")
                    && !row[0].starts_with('-')
            })
            .flat_map(|row| {
                row[0].split(", ").map(move |label| {
                    let locale = label.split(" (").next().unwrap();
                    SupportedLocales::new(
                        Locale::from_code(locale).expect("documented locale"),
                        row[1],
                        (row[2] != "Not selected")
                            .then(|| Market::from_code(row[2]).expect("documented locale context")),
                        Language::from_code(row[3]).expect("documented language"),
                    )
                })
            })
            .collect();
        // Tables group entries by evidence notes, which now exist only as comments.
        // Compare every modeled value, retaining duplicate entries.
        let key = |locale: &SupportedLocales| {
            (
                locale.locale.as_str(),
                locale.url.clone(),
                locale.country_region.map(Market::as_str),
                locale.language.as_str(),
            )
        };
        let mut actual = retailer.supported_locales;
        actual.sort_by_key(key);
        expected.sort_by_key(key);
        assert_eq!(actual, expected, "{code}");
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
    assert_eq!(eu.target_markets, [Market::Region(Region::EuropeanUnion)]);
    assert_eq!(eu.supported_locales[0].locale, Locale::en_EU);
    assert_eq!(
        eu.supported_locales[0].country_region,
        Some(Market::Country(Country::Austria))
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
    assert_eq!(germany.supported_locales[1].locale, Locale::de_DE);
    assert!(
        anker::get_retailer("ankerkr")
            .unwrap()
            .shopify_shop
            .is_none()
    );
    let main = ugreen::get_retailer("ugreencom").unwrap();
    assert_eq!(main.supported_locales().len(), 3);
    assert!(
        main.supported_locales()
            .iter()
            .all(|locale| locale.country_region.is_some())
    );
    assert_eq!(
        get_retailer(RetailerCode::AnkerCa),
        anker::get_retailer("ankerca")
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
        assert_eq!(
            retailer.target_markets,
            [Market::from_code(country).unwrap()]
        );
        assert_eq!(retailer.storefront_website, legacy.storefront_website);
        assert_eq!(retailer.shopify_shop, legacy.shopify_shop);
        assert_eq!(
            retailer.storefront_graphql_api,
            legacy.storefront_graphql_api
        );
        assert_eq!(retailer.shipping_countries, legacy.shipping_countries);
        assert_eq!(retailer.supported_locales.len(), 1);
        assert_eq!(
            retailer.supported_locales[0].country_region,
            Market::from_code(country)
        );
        assert_eq!(retailer.supported_locales[0].language, Language::EN);
    }
}

#[test]
fn market_codes_preserve_country_and_region_identity() {
    assert_eq!(
        Market::from_code("EU"),
        Some(Market::Region(Region::EuropeanUnion))
    );
    assert_eq!(
        Market::from_code("AT"),
        Some(Market::Country(Country::Austria))
    );
    assert!(Country::from_code("EU").is_none());
    assert!(Region::from_code("AT").is_none());
    for code in ["", "eu", "at", "AT ", " EU", "XX"] {
        assert!(Market::from_code(code).is_none());
    }
    let inventory = include_str!("../../../../docs/retailer.md");
    for line in inventory
        .lines()
        .filter(|line| line.starts_with("| Retailer code"))
    {
        let code = line.split('|').nth(2).unwrap().trim().trim_matches('`');
        let retailer = FACTORIES.iter().find_map(|factory| factory(code)).unwrap();
        for market in retailer
            .target_markets
            .into_iter()
            .chain(retailer.shipping_countries)
            .chain(
                retailer
                    .supported_locales
                    .into_iter()
                    .filter_map(|locale| locale.country_region),
            )
        {
            assert_eq!(Market::from_code(market.as_str()), Some(market));
            match market {
                Market::Country(country) => {
                    assert_eq!(Country::from_code(country.as_str()), Some(country));
                    assert!(Region::from_code(country.as_str()).is_none());
                }
                Market::Region(region) => {
                    assert_eq!(Region::from_code(region.as_str()), Some(region));
                    assert!(Country::from_code(region.as_str()).is_none());
                }
            }
        }
    }
}

#[test]
fn locale_tags_round_trip_with_canonical_casing() {
    let inventory = include_str!("../../../../docs/retailer.md");
    let mut locales = std::collections::HashSet::new();
    for row in inventory.lines().filter(|line| line.starts_with('|')) {
        let cells: Vec<_> = row.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() != 4
            || matches!(cells[0], "Supported locales" | "Advertised-only locales")
            || cells[0].starts_with('-')
        {
            continue;
        }
        for label in cells[0].split(", ") {
            let tag = label.split(" (").next().unwrap();
            let locale = Locale::from_code(tag).expect("known inventory locale");
            assert!(locale.as_str().eq_ignore_ascii_case(tag));
            assert_eq!(Locale::from_code(locale.as_str()), Some(locale));
            locales.insert(locale);
        }
    }
    assert_eq!(locales.len(), 106);
    for (tag, expected) in [
        ("en-dk", Locale::en_DK),
        ("EN-fi", Locale::en_FI),
        ("en-is", Locale::en_IS),
        ("en-no", Locale::en_NO),
        ("ZH-tw", Locale::zh_TW),
    ] {
        assert_eq!(Locale::from_code(tag), Some(expected));
    }
    assert_eq!(Locale::en_DK.as_str(), "en-DK");
    assert_eq!(Locale::zh_TW.as_str(), "zh-TW");
    assert_ne!(Locale::en, Locale::en_US);
    for tag in ["", "xx", "en-XX", "en_US", " en-US", "en-US "] {
        assert!(Locale::from_code(tag).is_none());
    }
}
