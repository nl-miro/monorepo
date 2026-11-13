//! Retailer inventory from `docs/retailer.md`.

use super::{Country, Language, Locale, Market, Region, Retailer, RetailerCode, SupportedLocales};

/// Returns the documented retailer for an exact, lowercase retailer code.
pub fn get_retailer(retailer_code: &str) -> Option<Retailer> {
    match retailer_code {
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-canada).
        // Keep this record synchronized with its inventory tables.
        "ugreenca" => Some(Retailer::new(
            RetailerCode::UgreenCa,
            "https://ca.ugreen.com/",
            Some("ugreen-ca.myshopify.com".into()),
            Some("https://ugreen-ca.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Canada)],
            vec![],
            None,
            vec![
                SupportedLocales::new(
                    Locale::en,
                    "https://ca.ugreen.com/",
                    Some(Market::Country(Country::Canada)),
                    Language::EN,
                ),
                SupportedLocales::new(
                    // context verified 2026-11-13
                    Locale::fr,
                    "https://ca.ugreen.com/fr",
                    Some(Market::Country(Country::Canada)),
                    Language::FR,
                ),
            ],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-germany).
        // Keep this record synchronized with its inventory tables.
        "ugreende" => Some(Retailer::new(
            RetailerCode::UgreenDe,
            "https://de.ugreen.com/",
            Some("ugreen-de.myshopify.com".into()),
            Some("https://ugreen-de.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Germany)],
            [
                Country::Germany,
                Country::Austria,
                Country::Denmark,
                Country::France,
                Country::Netherlands,
                Country::Poland,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                Locale::de,
                "https://de.ugreen.com/",
                Some(Market::Country(Country::Germany)),
                Language::DE,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-europe).
        // Keep this record synchronized with its inventory tables.
        "ugreeneu" => Some(Retailer::new(
            RetailerCode::UgreenEu,
            "https://eu.ugreen.com/",
            Some("ugreen-eu.myshopify.com".into()),
            Some("https://ugreen-eu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Region(Region::EuropeanUnion)],
            [
                Country::Bulgaria,
                Country::Croatia,
                Country::Estonia,
                Country::Hungary,
                Country::Portugal,
                Country::Romania,
                Country::Slovenia,
                Country::Sweden,
                Country::Austria,
                Country::Belgium,
                Country::Czechia,
                Country::France,
                Country::Germany,
                Country::Italy,
                Country::Malta,
                Country::Netherlands,
                Country::Poland,
                Country::Spain,
                Country::Denmark,
                Country::Latvia,
                Country::Finland,
                Country::Luxembourg,
                Country::Slovakia,
                Country::Cyprus,
                Country::Lithuania,
                Country::Switzerland,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![
                SupportedLocales::new(Locale::en, "https://eu.ugreen.com/", None, Language::EN),
                SupportedLocales::new(
                    // context verified 2026-11-13
                    Locale::bg_BG,
                    "https://eu.ugreen.com/bg-bg",
                    Some(Market::Country(Country::Bulgaria)),
                    Language::BG,
                ),
                SupportedLocales::new(
                    // context verified 2026-11-13
                    Locale::cs_CZ,
                    "https://eu.ugreen.com/cs-cz",
                    Some(Market::Country(Country::Czechia)),
                    Language::CS,
                ),
                SupportedLocales::new(
                    // context verified 2026-11-13
                    Locale::da_DK,
                    "https://eu.ugreen.com/da-dk",
                    Some(Market::Country(Country::Denmark)),
                    Language::DA,
                ),
                SupportedLocales::new(
                    // context verified 2026-11-13
                    Locale::de_AT,
                    "https://eu.ugreen.com/de-at",
                    Some(Market::Country(Country::Austria)),
                    Language::DE,
                ),
            ],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-france).
        // Keep this record synchronized with its inventory tables.
        "ugreenfr" => Some(Retailer::new(
            RetailerCode::UgreenFr,
            "https://fr.ugreen.com/",
            Some("ugreen-fr.myshopify.com".into()),
            Some("https://ugreen-fr.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::France)],
            [
                Country::France,
                Country::Austria,
                Country::Belgium,
                Country::Germany,
                Country::Italy,
                Country::Spain,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                Locale::fr,
                "https://fr.ugreen.com/",
                Some(Market::Country(Country::France)),
                Language::FR,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-south-korea).
        // Keep this record synchronized with its inventory tables.
        "ugreenkr" => Some(Retailer::new(
            RetailerCode::UgreenKr,
            "https://kr.ugreen.com/",
            Some("ugreen-kr.myshopify.com".into()),
            Some("https://ugreen-kr.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::SouthKorea)],
            vec![],
            None,
            vec![SupportedLocales::new(
                Locale::ko,
                "https://kr.ugreen.com/",
                Some(Market::Country(Country::SouthKorea)),
                Language::KO,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-nas-australia).
        // Keep this record synchronized with its inventory tables.
        "ugreennasau" => Some(Retailer::new(
            RetailerCode::UgreenNasAu,
            "https://nas-au.ugreen.com/",
            Some("nas-ugreen-au.myshopify.com".into()),
            Some("https://nas-ugreen-au.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Australia)],
            vec![],
            None,
            vec![SupportedLocales::new(
                Locale::en,
                "https://nas-au.ugreen.com/",
                Some(Market::Country(Country::Australia)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-nas-canada).
        // Keep this record synchronized with its inventory tables.
        "ugreennasca" => Some(Retailer::new(
            RetailerCode::UgreenNasCa,
            "https://nas-ca.ugreen.com/",
            Some("ugreen-nas-ca.myshopify.com".into()),
            Some("https://ugreen-nas-ca.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Canada)],
            vec![Market::Country(Country::Canada)],
            None,
            vec![SupportedLocales::new(
                Locale::en,
                "https://nas-ca.ugreen.com/",
                Some(Market::Country(Country::Canada)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-nas-germany).
        // Keep this record synchronized with its inventory tables.
        "ugreennasde" => Some(Retailer::new(
            RetailerCode::UgreenNasDe,
            "https://nas-de.ugreen.com/",
            Some("ugreen-de-nas.myshopify.com".into()),
            Some("https://ugreen-de-nas.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Germany)],
            vec![
                Market::Country(Country::Germany),
                Market::Country(Country::Austria),
            ],
            None,
            vec![SupportedLocales::new(
                // selector
                Locale::de,
                "https://nas-de.ugreen.com/",
                Some(Market::Country(Country::Germany)),
                Language::DE,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-nas-spain).
        // Keep this record synchronized with its inventory tables.
        "ugreennases" => Some(Retailer::new(
            RetailerCode::UgreenNasEs,
            "https://nas-es.ugreen.com/",
            Some("ugreen-nas-es.myshopify.com".into()),
            Some("https://ugreen-nas-es.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Spain)],
            vec![Market::Country(Country::Spain)],
            None,
            vec![SupportedLocales::new(
                // selector
                Locale::es,
                "https://nas-es.ugreen.com/",
                Some(Market::Country(Country::Spain)),
                Language::ES,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-nas-europe).
        // Keep this record synchronized with its inventory tables.
        "ugreennaseu" => Some(Retailer::new(
            RetailerCode::UgreenNasEu,
            "https://nas-eu.ugreen.com/",
            Some("ugreen-nas-eu.myshopify.com".into()),
            Some("https://ugreen-nas-eu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Region(Region::EuropeanUnion)],
            [
                Country::Germany,
                Country::France,
                Country::Italy,
                Country::Netherlands,
                Country::Spain,
                Country::Belgium,
                Country::Portugal,
                Country::Romania,
                Country::Czechia,
                Country::Poland,
                Country::Croatia,
                Country::Austria,
                Country::Bulgaria,
                Country::Lithuania,
                Country::Ireland,
                Country::Sweden,
                Country::Greece,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                // selector
                Locale::en,
                "https://nas-eu.ugreen.com/",
                None,
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-nas-france).
        // Keep this record synchronized with its inventory tables.
        "ugreennasfr" => Some(Retailer::new(
            RetailerCode::UgreenNasFr,
            "https://nas-fr.ugreen.com/",
            Some("nas-ugreen-fr.myshopify.com".into()),
            Some("https://nas-ugreen-fr.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::France)],
            vec![Market::Country(Country::France)],
            None,
            vec![SupportedLocales::new(
                // selector
                Locale::fr,
                "https://nas-fr.ugreen.com/",
                Some(Market::Country(Country::France)),
                Language::FR,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-nas-italy).
        // Keep this record synchronized with its inventory tables.
        "ugreennasit" => Some(Retailer::new(
            RetailerCode::UgreenNasIt,
            "https://nas-it.ugreen.com/",
            Some("ugreen-nas-it.myshopify.com".into()),
            Some("https://ugreen-nas-it.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Italy)],
            vec![Market::Country(Country::Italy)],
            None,
            vec![SupportedLocales::new(
                // selector
                Locale::it,
                "https://nas-it.ugreen.com/",
                Some(Market::Country(Country::Italy)),
                Language::IT,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-nas-united-kingdom).
        // Keep this record synchronized with its inventory tables.
        "ugreennasuk" => Some(Retailer::new(
            RetailerCode::UgreenNasUk,
            "https://nas-uk.ugreen.com/",
            Some("ugreen-nas-uk.myshopify.com".into()),
            Some("https://ugreen-nas-uk.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::UnitedKingdom)],
            vec![Market::Country(Country::UnitedKingdom)],
            None,
            vec![SupportedLocales::new(
                // selector
                Locale::en,
                "https://nas-uk.ugreen.com/",
                Some(Market::Country(Country::UnitedKingdom)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-nas-united-states).
        // Keep this record synchronized with its inventory tables.
        "ugreennas" => Some(Retailer::new(
            RetailerCode::UgreenNas,
            "https://nas.ugreen.com/",
            Some("ugreen-nas.myshopify.com".into()),
            Some("https://ugreen-nas.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::UnitedStates)],
            vec![Market::Country(Country::UnitedStates)],
            None,
            vec![SupportedLocales::new(
                Locale::en,
                "https://nas.ugreen.com/",
                Some(Market::Country(Country::UnitedStates)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-nas-japan).
        // Keep this record synchronized with its inventory tables.
        "ugreennasjp" => Some(Retailer::new(
            RetailerCode::UgreenNasJp,
            "https://nas.ugreen.jp/",
            Some("ugreen-nas-jp.myshopify.com".into()),
            Some("https://ugreen-nas-jp.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Japan)],
            vec![Market::Country(Country::Japan)],
            None,
            vec![SupportedLocales::new(
                // selector
                Locale::ja,
                "https://nas.ugreen.jp/",
                Some(Market::Country(Country::Japan)),
                Language::JA,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-netherlands).
        // Keep this record synchronized with its inventory tables.
        "ugreennl" => Some(Retailer::new(
            RetailerCode::UgreenNl,
            "https://nl.ugreen.com/",
            Some("ugreen-nl.myshopify.com".into()),
            Some("https://ugreen-nl.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Netherlands)],
            [
                Country::Netherlands,
                Country::Belgium,
                Country::France,
                Country::Germany,
                Country::Spain,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                // selector
                Locale::nl,
                "https://nl.ugreen.com/",
                Some(Market::Country(Country::Netherlands)),
                Language::NL,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-japan).
        // Keep this record synchronized with its inventory tables.
        "ugreenjp" => Some(Retailer::new(
            RetailerCode::UgreenJp,
            "https://ugreen.jp/",
            Some("ugreen-jp.myshopify.com".into()),
            Some("https://ugreen-jp.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Japan)],
            vec![],
            None,
            vec![SupportedLocales::new(
                Locale::ja,
                "https://ugreen.jp/",
                Some(Market::Country(Country::Japan)),
                Language::JA,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-united-kingdom).
        // Keep this record synchronized with its inventory tables.
        "ugreenuk" => Some(Retailer::new(
            RetailerCode::UgreenUk,
            "https://uk.ugreen.com/",
            Some("ugreen-uk.myshopify.com".into()),
            Some("https://ugreen-uk.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::UnitedKingdom)],
            vec![Market::Country(Country::UnitedKingdom)],
            Some("Shipping query verified 2026-11-13; storefront confirms backend.".into()),
            vec![SupportedLocales::new(
                // selector
                Locale::en,
                "https://uk.ugreen.com/",
                Some(Market::Country(Country::UnitedKingdom)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-united-states).
        // Keep this record synchronized with its inventory tables.
        "ugreenus" => Some(Retailer::new(
            RetailerCode::UgreenUs,
            "https://us.ugreen.com/",
            Some("ugreenmall.myshopify.com".into()),
            Some("https://ugreenmall.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::UnitedStates)],
            vec![Market::Country(Country::UnitedStates)],
            Some("Shipping query verified 2026-11-13; storefront confirms backend.".into()),
            vec![SupportedLocales::new(
                // selector
                Locale::en,
                "https://us.ugreen.com/",
                Some(Market::Country(Country::UnitedStates)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-main-site).
        // Keep this record synchronized with its inventory tables.
        "ugreencom" => Some(Retailer::new(
            RetailerCode::UgreenCom,
            "https://www.ugreen.com/",
            Some("ugreen-brandsite.myshopify.com".into()),
            Some("https://ugreen-brandsite.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::UnitedStates)],
            vec![],
            None,
            vec![
                SupportedLocales::new(
                    Locale::en,
                    "https://www.ugreen.com/",
                    Some(Market::Country(Country::UnitedStates)),
                    Language::EN,
                ),
                SupportedLocales::new(
                    Locale::en_US,
                    "https://www.ugreen.com/",
                    Some(Market::Country(Country::UnitedStates)),
                    Language::EN,
                ),
                SupportedLocales::new(
                    // context verified 2026-11-13
                    Locale::en_CA,
                    "https://www.ugreen.com/en-ca",
                    Some(Market::Country(Country::Canada)),
                    Language::EN,
                ),
            ],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#ugreen-india).
        // Keep this record synchronized with its inventory tables.
        "ugreenin" => Some(Retailer::new(
            RetailerCode::UgreenIn,
            "https://www.ugreenindia.com/",
            Some("990f9c-b7.myshopify.com".into()),
            Some("https://990f9c-b7.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::India)],
            vec![Market::Country(Country::India)],
            None,
            vec![SupportedLocales::new(
                Locale::en,
                "https://www.ugreenindia.com/",
                Some(Market::Country(Country::India)),
                Language::EN,
            )],
        )),
        _ => None,
    }
}
