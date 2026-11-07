use everyday::prelude::CountryCode;
use retailer_sauce::localization::RegionCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TargetMarket {
    Country { code: CountryCode },
    Region { code: RegionCode },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn country_and_region_codes_round_trip() {
        let country = TargetMarket::Country {
            code: CountryCode::HR,
        };
        let region = TargetMarket::Region {
            code: RegionCode::EuropeanUnion,
        };
        for (market, json) in [
            (country, json!({"kind": "country", "code": "HR"})),
            (region, json!({"kind": "region", "code": "eu"})),
        ] {
            assert_eq!(serde_json::to_value(market).unwrap(), json);
            assert_eq!(
                serde_json::from_value::<TargetMarket>(json).unwrap(),
                market
            );
        }
    }

    #[test]
    fn rejects_legacy_labels_and_invalid_codes() {
        for json in [
            json!({"kind": "region", "label": "European Union"}),
            json!({"kind": "region", "code": "eu", "label": "European Union"}),
            json!({"kind": "country", "code": "HR", "label": "Croatia"}),
            json!({"kind": "country", "code": "XX"}),
            json!({"kind": "country", "code": "eu"}),
            json!({"kind": "region", "code": "HR"}),
            json!({"kind": "region", "code": "unknown"}),
            json!({"kind": "region"}),
            json!({"kind": "country", "code": null}),
            json!({"kind": "legacy_region", "label": "Europe"}),
        ] {
            assert!(serde_json::from_value::<TargetMarket>(json).is_err());
        }
    }
}
