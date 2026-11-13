//! Retailer inventory from `docs/retailer.md`.

use super::{Country, Language, Locale, Market, Region, Retailer, RetailerCode, SupportedLocales};

/// Returns the documented retailer for an exact, lowercase retailer code.
pub fn get_retailer(retailer_code: &str) -> Option<Retailer> {
    match retailer_code {
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-australia).
        // Keep this record synchronized with its inventory tables.
        "minisforumau" => Some(Retailer::new(
            RetailerCode::MinisForumAu,
            "https://au.minisforum.com/",
            Some("j94nqi-71.myshopify.com".into()),
            Some("https://j94nqi-71.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Australia)],
            vec![Market::Country(Country::Australia)],
            None,
            vec![SupportedLocales::new(
                Locale::en,
                "https://au.minisforum.com/",
                Some(Market::Country(Country::Australia)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-canada).
        // Keep this record synchronized with its inventory tables.
        "minisforumca" => Some(Retailer::new(
            RetailerCode::MinisForumCa,
            "https://ca.minisforum.com/",
            Some("seaminisforum.myshopify.com".into()),
            Some("https://seaminisforum.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Canada)],
            vec![Market::Country(Country::Canada)],
            None,
            vec![SupportedLocales::new(
                Locale::en,
                "https://ca.minisforum.com/",
                Some(Market::Country(Country::Canada)),
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-hong-kong).
        // Keep this record synchronized with its inventory tables.
        "minisforumhk" => Some(Retailer::new(
            RetailerCode::MinisForumHk,
            "https://minisforum.hk/",
            Some("minisforumcn.myshopify.com".into()),
            Some("https://minisforumcn.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::HongKong)],
            vec![Market::Country(Country::HongKong)],
            None,
            vec![SupportedLocales::new(
                Locale::zh_TW,
                "https://minisforum.hk/",
                Some(Market::Country(Country::HongKong)),
                Language::ZH_TW,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-south-korea).
        // Keep this record synchronized with its inventory tables.
        "minisforumkr" => Some(Retailer::new(
            RetailerCode::MinisForumKr,
            "https://minisforum.kr/",
            Some("minisforumkr.myshopify.com".into()),
            Some("https://minisforumkr.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::SouthKorea)],
            vec![Market::Country(Country::SouthKorea)],
            None,
            vec![SupportedLocales::new(
                Locale::ko,
                "https://minisforum.kr/",
                Some(Market::Country(Country::SouthKorea)),
                Language::KO,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-russia).
        // Keep this record synchronized with its inventory tables.
        "minisforumru" => Some(Retailer::new(
            RetailerCode::MinisForumRu,
            "https://minisforum.ru/",
            Some("store-minisforum-ru.myshopify.com".into()),
            Some("https://store-minisforum-ru.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Russia)],
            vec![],
            None,
            vec![SupportedLocales::new(
                Locale::ru,
                "https://minisforum.ru/",
                Some(Market::Country(Country::Russia)),
                Language::RU,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-europe).
        // Keep this record synchronized with its inventory tables.
        "minisforumeu" => Some(Retailer::new(
            RetailerCode::MinisForumEu,
            "https://minisforumpc.eu/",
            Some("minisforum-eu.myshopify.com".into()),
            Some("https://minisforum-eu.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Region(Region::EuropeanUnion)],
            [
                Country::Czechia,
                Country::Denmark,
                Country::Austria,
                Country::Belgium,
                Country::Bulgaria,
                Country::Croatia,
                Country::Estonia,
                Country::Finland,
                Country::France,
                Country::Germany,
                Country::Greece,
                Country::Ireland,
                Country::Italy,
                Country::Latvia,
                Country::Lithuania,
                Country::Luxembourg,
                Country::Netherlands,
                Country::Poland,
                Country::Portugal,
                Country::Romania,
                Country::Slovakia,
                Country::Slovenia,
                Country::Spain,
                Country::Sweden,
                Country::Hungary,
                Country::Switzerland,
                Country::Albania,
                Country::Belarus,
                Country::NorthMacedonia,
                Country::Iceland,
                Country::BosniaAndHerzegovina,
                Country::VaticanCity,
                Country::Liechtenstein,
                Country::Monaco,
                Country::SanMarino,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![SupportedLocales::new(
                Locale::en,
                "https://minisforumpc.eu/",
                None,
                Language::EN,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-france).
        // Keep this record synchronized with its inventory tables.
        "minisforumfr" => Some(Retailer::new(
            RetailerCode::MinisForumFr,
            "https://minisforumpc.fr/",
            Some("minisforum-fr.myshopify.com".into()),
            Some("https://minisforum-fr.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::France)],
            vec![
                Market::Country(Country::Belgium),
                Market::Country(Country::France),
                Market::Country(Country::Luxembourg),
            ],
            None,
            vec![SupportedLocales::new(
                Locale::fr,
                "https://minisforumpc.fr/",
                Some(Market::Country(Country::France)),
                Language::FR,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-united-states).
        // Keep this record synchronized with its inventory tables.
        "minisforumus" => Some(Retailer::new(
            RetailerCode::MinisForumUs,
            "https://store.minisforum.com/",
            Some("minisforum.myshopify.com".into()),
            Some("https://minisforum.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::UnitedStates)],
            [
                Country::UnitedStates,
                Country::Canada,
                Country::UnitedArabEmirates,
                Country::Australia,
                Country::SaudiArabia,
                Country::Philippines,
                Country::NewZealand,
                Country::AntiguaAndBarbuda,
                Country::Bahamas,
                Country::Barbados,
                Country::Belize,
                Country::Bermuda,
                Country::CaymanIslands,
                Country::DominicanRepublic,
                Country::Ecuador,
                Country::ElSalvador,
                Country::Grenada,
                Country::Honduras,
                Country::Jamaica,
                Country::Liberia,
                Country::Mexico,
                Country::Nicaragua,
                Country::Panama,
                Country::SaintKittsAndNevis,
                Country::SaintLucia,
                Country::TrinidadAndTobago,
                Country::TurksAndCaicosIslands,
                Country::Venezuela,
                Country::Fiji,
                Country::PapuaNewGuinea,
                Country::Samoa,
                Country::SolomonIslands,
                Country::Tonga,
                Country::Vanuatu,
                Country::Singapore,
                Country::Indonesia,
                Country::Malaysia,
                Country::Israel,
                Country::Kuwait,
            ]
            .into_iter()
            .map(Market::Country)
            .collect(),
            None,
            vec![
                SupportedLocales::new(
                    Locale::en,
                    "https://store.minisforum.com/",
                    Some(Market::Country(Country::UnitedStates)),
                    Language::EN,
                ),
                SupportedLocales::new(
                    // context verified 2026-11-13
                    Locale::en_CA,
                    "https://store.minisforum.com/en-ca",
                    Some(Market::Country(Country::Canada)),
                    Language::EN,
                ),
                SupportedLocales::new(
                    // context verified 2026-11-13
                    Locale::en_PE,
                    "https://store.minisforum.com/en-os",
                    Some(Market::Country(Country::Peru)),
                    Language::EN,
                ),
            ],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-japan).
        // Keep this record synchronized with its inventory tables.
        "minisforumjp" => Some(Retailer::new(
            RetailerCode::MinisForumJp,
            "https://www.minisforum.jp/",
            Some("minisforum-jp.myshopify.com".into()),
            Some("https://minisforum-jp.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::Japan)],
            vec![Market::Country(Country::Japan)],
            None,
            vec![SupportedLocales::new(
                Locale::ja,
                "https://www.minisforum.jp/",
                Some(Market::Country(Country::Japan)),
                Language::JA,
            )],
        )),
        // Inventory: [docs/retailer.md](../../../../docs/retailer.md#minisforum-united-kingdom).
        // Keep this record synchronized with its inventory tables.
        "minisforumuk" => Some(Retailer::new(
            RetailerCode::MinisForumUk,
            "https://www.minisforum.uk/",
            Some("minisforum-uk.myshopify.com".into()),
            Some("https://minisforum-uk.myshopify.com/api/2026-10/graphql.json".into()),
            vec![Market::Country(Country::UnitedKingdom)],
            vec![
                Market::Country(Country::Ireland),
                Market::Country(Country::UnitedKingdom),
            ],
            None,
            vec![SupportedLocales::new(
                Locale::en,
                "https://www.minisforum.uk/",
                Some(Market::Country(Country::UnitedKingdom)),
                Language::EN,
            )],
        )),
        _ => None,
    }
}
