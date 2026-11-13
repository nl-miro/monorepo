//! Retailer inventory from `docs/retailer.md`.

use super::{Country, Language, Locale, Market, Region, Retailer, RetailerCode, SupportedLocales};

/// Returns the documented retailer for an exact, lowercase retailer code.
pub fn get_retailer(retailer_code: &str) -> Option<Retailer> {
    match retailer_code {
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-australia).
        // Keep this record synchronized with its inventory tables.
        "ankerau" => Some(Retailer::new(
            RetailerCode::AnkerAu,
            "https://www.anker.com/au",
            Some("oneanker-au.myshopify.com".into()),
            Some("https://oneanker-au.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Australia)],
            vec![Market::Country(Country::Australia)],
            None,
            vec![SupportedLocales::new(
                Locale::en_AU,
                "https://www.anker.com/au",
                Some(Market::Country(Country::Australia)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-canada).
        // Keep this record synchronized with its inventory tables.
        "ankerca" => Some(Retailer::new(
            RetailerCode::AnkerCa,
            "https://www.anker.com/ca",
            Some("anker-canada.myshopify.com".into()),
            Some("https://anker-canada.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Canada)],
            vec![Market::Country(Country::Canada)],
            None,
            vec![
                SupportedLocales::new(
                    Locale::en_CA,
                    "https://www.anker.com/ca",
                    Some(Market::Country(Country::Canada)),
                    Language::EN,
                ),
                SupportedLocales::new(
                    Locale::fr_CA,
                    "https://www.anker.com/ca-fr",
                    Some(Market::Country(Country::Canada)),
                    Language::FR,
                ),
            ],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-germany).
        // Keep this record synchronized with its inventory tables.
        "ankerde" => Some(Retailer::new(
            RetailerCode::AnkerDe,
            "https://www.anker.com/de",
            Some("oneanker-de.myshopify.com".into()),
            Some("https://oneanker-de.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Germany)],
            vec![Market::Country(Country::Germany)],
            None,
            vec![
                SupportedLocales::new(
                    // HTML
                    Locale::de,
                    "https://www.anker.com/de",
                    Some(Market::Country(Country::Germany)),
                    Language::DE,
                ),
                SupportedLocales::new(
                    // alternate link
                    Locale::de_DE,
                    "https://www.anker.com/de",
                    Some(Market::Country(Country::Germany)),
                    Language::DE,
                ),
            ],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-europe).
        // Keep this record synchronized with its inventory tables.
        "ankereu" => Some(Retailer::new(
            RetailerCode::AnkerEu,
            "https://www.anker.com/eu-en",
            Some("ankereu.myshopify.com".into()),
            Some("https://ankereu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Region(Region::EuropeanUnion)],
            [
                Country::Austria,
                Country::Belgium,
                Country::Bulgaria,
                Country::Czechia,
                Country::Germany,
                Country::Denmark,
                Country::Estonia,
                Country::Spain,
                Country::Finland,
                Country::France,
                Country::Greece,
                Country::Croatia,
                Country::Hungary,
                Country::Italy,
                Country::Lithuania,
                Country::Luxembourg,
                Country::Latvia,
                Country::Netherlands,
                Country::Poland,
                Country::Portugal,
                Country::Romania,
                Country::Sweden,
                Country::Slovenia,
                Country::Slovakia,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                Locale::en_EU,
                "https://www.anker.com/eu-en",
                Some(Market::Country(Country::Austria)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-south-korea).
        // Keep this record synchronized with its inventory tables.
        "ankerkr" => Some(Retailer::new(
            RetailerCode::AnkerKr,
            "https://ankerkorea.co.kr/",
            None,
            None,
            vec![Market::Country(Country::SouthKorea)],
            vec![],
            Some("Unknown; no endpoint established in the research".into()),
            vec![
                SupportedLocales::new(
                    Locale::ko,
                    "https://ankerkorea.co.kr/",
                    Some(Market::Country(Country::SouthKorea)),
                    Language::KO,
                ),
                SupportedLocales::new(
                    // alternate link
                    Locale::ko_KR,
                    "https://ankerkorea.co.kr/",
                    Some(Market::Country(Country::SouthKorea)),
                    Language::KO,
                ),
            ],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-united-states).
        // Keep this record synchronized with its inventory tables.
        "ankercom" => Some(Retailer::new(
            RetailerCode::AnkerCom,
            "https://www.anker.com/",
            Some("ankerus.myshopify.com".into()),
            Some("https://ankerus.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::UnitedStates)],
            vec![Market::Country(Country::UnitedStates)],
            None,
            vec![SupportedLocales::new(
                Locale::en_US,
                "https://www.anker.com/",
                Some(Market::Country(Country::UnitedStates)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-poland).
        // Keep this record synchronized with its inventory tables.
        "ankerpl" => Some(Retailer::new(
            RetailerCode::AnkerPl,
            "https://www.anker.com/eu-pl",
            Some("ankereu.myshopify.com".into()),
            Some("https://ankereu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Poland)],
            [
                Country::Austria,
                Country::Belgium,
                Country::Bulgaria,
                Country::Croatia,
                Country::Czechia,
                Country::Denmark,
                Country::Estonia,
                Country::Finland,
                Country::France,
                Country::Greece,
                Country::Spain,
                Country::Netherlands,
                Country::Lithuania,
                Country::Luxembourg,
                Country::Germany,
                Country::Poland,
                Country::Portugal,
                Country::Romania,
                Country::Sweden,
                Country::Slovakia,
                Country::Slovenia,
                Country::Hungary,
                Country::Italy,
                Country::Latvia,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                Locale::pl_PL,
                "https://www.anker.com/eu-pl",
                Some(Market::Country(Country::Poland)),
                Language::PL,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-france).
        // Keep this record synchronized with its inventory tables.
        "ankerfr" => Some(Retailer::new(
            RetailerCode::AnkerFr,
            "https://www.anker.com/fr",
            Some("ankerfra.myshopify.com".into()),
            Some("https://ankerfra.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::France)],
            vec![
                Market::Country(Country::France),
                Market::Country(Country::Italy),
                Market::Country(Country::Spain),
            ],
            None,
            vec![SupportedLocales::new(
                Locale::fr_FR,
                "https://www.anker.com/fr",
                Some(Market::Country(Country::France)),
                Language::FR,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-new-zealand).
        // Keep this record synchronized with its inventory tables.
        "ankernz" => Some(Retailer::new(
            RetailerCode::AnkerNz,
            "https://www.anker.com/nz",
            Some("ankernz.myshopify.com".into()),
            Some("https://ankernz.myshopify.com/api/2026-10/graphql.json".into()),
            vec![],
            vec![],
            None,
            vec![SupportedLocales::new(
                Locale::en_NZ,
                "https://www.anker.com/nz",
                Some(Market::Country(Country::NewZealand)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-united-kingdom).
        // Keep this record synchronized with its inventory tables.
        "ankeruk" => Some(Retailer::new(
            RetailerCode::AnkerUk,
            "https://www.anker.com/uk",
            Some("oneanker-uk.myshopify.com".into()),
            Some("https://oneanker-uk.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::UnitedKingdom)],
            vec![Market::Country(Country::UnitedKingdom)],
            None,
            vec![SupportedLocales::new(
                Locale::en_GB,
                "https://www.anker.com/uk",
                Some(Market::Country(Country::UnitedKingdom)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-vietnam).
        // Keep this record synchronized with its inventory tables.
        "ankervn" => Some(Retailer::new(
            RetailerCode::AnkerVn,
            "https://www.anker.com/vn",
            Some("ankervnm.myshopify.com".into()),
            Some("https://ankervnm.myshopify.com/api/2026-10/graphql.json".into()),
            vec![],
            vec![],
            Some("Responded with GraphQL error: Online Store channel is locked.".into()),
            vec![SupportedLocales::new(
                Locale::vi_VN,
                "https://www.anker.com/vn",
                Some(Market::Country(Country::Vietnam)),
                Language::VI,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-italy).
        // Keep this record synchronized with its inventory tables.
        "ankeritalycom" => Some(Retailer::new(
            RetailerCode::AnkerItalyCom,
            "https://www.ankeritaly.com/",
            Some("anker-italy.myshopify.com".into()),
            Some("https://anker-italy.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Italy)],
            [
                Country::Italy,
                Country::France,
                Country::Spain,
                Country::Germany,
                Country::Malta,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                Locale::it_IT,
                "https://www.ankeritaly.com/",
                Some(Market::Country(Country::Italy)),
                Language::IT,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-japan).
        // Keep this record synchronized with its inventory tables.
        "ankerjapancom" => Some(Retailer::new(
            RetailerCode::AnkerJapanCom,
            "https://www.ankerjapan.com/",
            Some("anker-japan-official-site.myshopify.com".into()),
            Some("https://anker-japan-official-site.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Japan)],
            vec![Market::Country(Country::Japan)],
            None,
            vec![SupportedLocales::new(
                Locale::ja,
                "https://www.ankerjapan.com/",
                Some(Market::Country(Country::Japan)),
                Language::JA,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-nordics).
        // Keep this record synchronized with its inventory tables.
        // Historical combined identity; new work uses the five country codes below.
        "ankernordicscom" => Some(Retailer::new(
            RetailerCode::AnkerNordicsCom,
            "https://www.ankernordics.com/",
            Some("ankerneu.myshopify.com".into()),
            Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![
                Market::Country(Country::Denmark),
                Market::Country(Country::Finland),
                Market::Country(Country::Iceland),
                Market::Country(Country::Norway),
                Market::Country(Country::Sweden),
            ],
            [
                Country::Sweden,
                Country::Norway,
                Country::Denmark,
                Country::Finland,
                Country::Iceland,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![
                SupportedLocales::new(
                    Locale::en_SE,
                    "https://www.ankernordics.com/",
                    Some(Market::Country(Country::Sweden)),
                    Language::EN,
                ),
                SupportedLocales::new(
                    // alternate link
                    Locale::en_DK,
                    "https://www.ankernordics.com/",
                    Some(Market::Country(Country::Denmark)),
                    Language::EN,
                ),
                SupportedLocales::new(
                    // alternate link
                    Locale::en_FI,
                    "https://www.ankernordics.com/",
                    Some(Market::Country(Country::Finland)),
                    Language::EN,
                ),
                SupportedLocales::new(
                    // alternate link
                    Locale::en_IS,
                    "https://www.ankernordics.com/",
                    Some(Market::Country(Country::Iceland)),
                    Language::EN,
                ),
                SupportedLocales::new(
                    // alternate link
                    Locale::en_NO,
                    "https://www.ankernordics.com/",
                    Some(Market::Country(Country::Norway)),
                    Language::EN,
                ),
            ],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-sweden).
        // Keep this record synchronized with its inventory tables.
        "ankerse" => Some(Retailer::new(
            RetailerCode::AnkerSe,
            "https://www.ankernordics.com/",
            Some("ankerneu.myshopify.com".into()),
            Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Sweden)],
            [
                Country::Sweden,
                Country::Norway,
                Country::Denmark,
                Country::Finland,
                Country::Iceland,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                Locale::en_SE,
                "https://www.ankernordics.com/",
                Some(Market::Country(Country::Sweden)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-norway).
        // Keep this record synchronized with its inventory tables.
        "ankerno" => Some(Retailer::new(
            RetailerCode::AnkerNo,
            "https://www.ankernordics.com/",
            Some("ankerneu.myshopify.com".into()),
            Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Norway)],
            [
                Country::Sweden,
                Country::Norway,
                Country::Denmark,
                Country::Finland,
                Country::Iceland,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                // alternate link
                Locale::en_NO,
                "https://www.ankernordics.com/",
                Some(Market::Country(Country::Norway)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-finland).
        // Keep this record synchronized with its inventory tables.
        "ankerfi" => Some(Retailer::new(
            RetailerCode::AnkerFi,
            "https://www.ankernordics.com/",
            Some("ankerneu.myshopify.com".into()),
            Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Finland)],
            [
                Country::Sweden,
                Country::Norway,
                Country::Denmark,
                Country::Finland,
                Country::Iceland,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                // alternate link
                Locale::en_FI,
                "https://www.ankernordics.com/",
                Some(Market::Country(Country::Finland)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-iceland).
        // Keep this record synchronized with its inventory tables.
        "ankeris" => Some(Retailer::new(
            RetailerCode::AnkerIs,
            "https://www.ankernordics.com/",
            Some("ankerneu.myshopify.com".into()),
            Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Iceland)],
            [
                Country::Sweden,
                Country::Norway,
                Country::Denmark,
                Country::Finland,
                Country::Iceland,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                // alternate link
                Locale::en_IS,
                "https://www.ankernordics.com/",
                Some(Market::Country(Country::Iceland)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#anker-denmark).
        // Keep this record synchronized with its inventory tables.
        "ankerdk" => Some(Retailer::new(
            RetailerCode::AnkerDk,
            "https://www.ankernordics.com/",
            Some("ankerneu.myshopify.com".into()),
            Some("https://ankerneu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Denmark)],
            [
                Country::Sweden,
                Country::Norway,
                Country::Denmark,
                Country::Finland,
                Country::Iceland,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                // alternate link
                Locale::en_DK,
                "https://www.ankernordics.com/",
                Some(Market::Country(Country::Denmark)),
                Language::EN,
            )],
        )),
        _ => None,
    }
}
