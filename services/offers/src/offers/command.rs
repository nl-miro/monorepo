use chrono::DateTime;
use chrono::Utc;
use retailer_sauce::prelude::Availability;
use retailer_sauce::prelude::ExternalIdentifiers;
use retailer_sauce::prelude::Gtin;
use retailer_sauce::prelude::ItemCondition;
use retailer_sauce::prelude::Localization;
use retailer_sauce::prelude::ProductIdentifiers;
use retailer_sauce::prelude::Sku;
use retailer_sauce::prelude::StructuralizedContent;

use kernel::ApplicationCommand;
use money::Money;
use retailer_sauce::prelude::Observation;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;

pub enum ProductOfferCommand {
    ImportOfferDetails(ImportOfferDetailsCommand),
}

#[derive(Serialize, Deserialize)]
pub struct ImportOfferDetailsCommand {
    #[serde(
        default,
        skip_serializing_if = "Observation::is_missing",
        deserialize_with = "retailer_sauce::prelude::object_observation"
    )]
    pub was_price: Observation<Money>,
    #[serde(default, skip_serializing_if = "Observation::is_missing")]
    pub availability: Observation<Availability>,
    #[serde(default, skip_serializing_if = "Observation::is_missing")]
    pub item_condition: Observation<ItemCondition>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "retailer_sauce::prelude::structuralized::observation"
    )]
    pub content: Option<StructuralizedContent>,

    #[serde(default)]
    pub gtin: Option<Gtin>,
    #[serde(default)]
    pub sku: Option<Sku>,
    #[serde(default)]
    pub external_identifiers: ExternalIdentifiers,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "retailer_sauce::prelude::localization::observation"
    )]
    pub localization: Option<Localization>,
    #[serde(default)]
    pub acquired_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub images: Vec<String>,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
}

pub const CREATE_FROM_OFFER_DETAILS_COMMAND: &str = "ImportOfferDetails";

impl ApplicationCommand for ImportOfferDetailsCommand {
    fn command_type(&self) -> &'static str {
        CREATE_FROM_OFFER_DETAILS_COMMAND
    }
}

#[cfg(test)]
mod export_contract_tests {
    use money::Currency;
    use serde_json::Value;
    use serde_json::from_str;
    use serde_json::from_value;
    use serde_json::json as serde_json_json;

    use super::*;

    #[test]
    fn accepts_the_sourcing_export_contract_fixture() {
        let fixture = include_str!(
            "../../../retailer_sourcing/tests/fixtures/offer_export/import_payload.json"
        );
        let command: ImportOfferDetailsCommand = from_str(fixture).unwrap();
        assert_eq!(command.command_type(), "ImportOfferDetails");
        assert_eq!(command.retailer_code, RetailerCode::AnkerAu);
        assert_eq!(command.title, " USB charger ž ");
        assert_eq!(command.price, Money::new(1999, Currency::AUD));
        assert_eq!(
            command.url,
            "https://shop.example/Charger?q=red&variant=1#details"
        );
        assert_eq!(
            command.acquired_at,
            Some("2026-10-26T12:00:00.123456Z".parse().unwrap())
        );
        assert_eq!(command.images, ["https://example.com/a.png"]);
        let localization = command.localization.unwrap();
        assert_eq!(localization.language.as_deref(), Some("en-AU"));
        assert_eq!(command.gtin.unwrap().value, "0012345678905");
        assert_eq!(command.sku.unwrap().value, "A1-BLACK");
        assert_eq!(command.was_price.value().unwrap().amount(), 2999);
        assert!(!command.availability.is_missing());
        assert!(!command.item_condition.is_missing());
        assert!(command.content.is_some());
    }

    #[test]
    fn a_manual_import_may_omit_the_acquisition_time() {
        let mut input: Value = from_str(include_str!(
            "../../../retailer_sourcing/tests/fixtures/offer_export/import_payload.json"
        ))
        .unwrap();
        input.as_object_mut().unwrap().remove("acquired_at");
        let command: ImportOfferDetailsCommand = from_value(input.clone()).unwrap();
        assert!(command.acquired_at.is_none());
        input["acquired_at"] = serde_json_json!("yesterday");
        assert!(from_value::<ImportOfferDetailsCommand>(input).is_err());
    }

    #[test]
    fn imports_have_strict_array_types() {
        let mut input: Value = from_str(include_str!(
            "../../../retailer_sourcing/tests/fixtures/offer_export/import_payload.json"
        ))
        .unwrap();
        input.as_object_mut().unwrap().remove("images");
        assert!(
            from_value::<ImportOfferDetailsCommand>(input.clone())
                .unwrap()
                .images
                .is_empty()
        );
        input["images"] = serde_json_json!(["https://shop.example/a", "/invalid"]);
        assert_eq!(
            from_value::<ImportOfferDetailsCommand>(input.clone())
                .unwrap()
                .images
                .len(),
            2
        );
        for value in [
            serde_json_json!(null),
            serde_json_json!("url"),
            serde_json_json!([3]),
        ] {
            input["images"] = value;
            assert!(from_value::<ImportOfferDetailsCommand>(input.clone()).is_err());
        }
    }
}

#[derive(Default)]
pub struct OfferObservation {
    pub was_price: Observation<Money>,
    pub availability: Observation<Availability>,
    pub item_condition: Observation<ItemCondition>,
    pub content: Option<StructuralizedContent>,
    pub identifiers: ProductIdentifiers,
    pub acquired_at: Option<DateTime<Utc>>,
    pub localization: Option<Localization>,
}
impl OfferObservation {
    pub fn guard(&self) -> Result<(), String> {
        if let Some(value) = self.availability.value() {
            value.guard()?;
        }
        if let Some(value) = self.item_condition.value() {
            value.guard()?;
        }
        if let Some(value) = &self.content {
            value.guard()?;
        }
        self.identifiers.guard()?;
        if let Some(localization) = &self.localization {
            localization.guard()?;
        }
        Ok(())
    }
}
