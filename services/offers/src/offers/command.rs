use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use kernel::ApplicationCommand;
use retailer_sauce::availability::Availability;
use retailer_sauce::item_condition::ItemCondition;
use serde::{Deserialize, Serialize};

use super::state::{Content, ExternalOfferIdentifiers, Price};
use super::target_market::TargetMarket;

pub enum ProductOfferCommand {
    ImportOfferDetails(ImportOfferDetailsCommand),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportOfferDetailsCommand {
    pub retailer_code: RetailerCode,
    pub url: String,
    pub identifiers: ExternalOfferIdentifiers,
    pub target_market: TargetMarket,
    pub language: String,
    pub content: Content,
    pub price: Price,
    pub availability: Option<Availability>,
    pub item_condition: Option<ItemCondition>,
    pub created_at: DateTime<Utc>,
}

pub const CREATE_FROM_OFFER_DETAILS_COMMAND: &str = "ImportOfferDetails";

impl ApplicationCommand for ImportOfferDetailsCommand {
    fn command_type(&self) -> &'static str {
        CREATE_FROM_OFFER_DETAILS_COMMAND
    }
}

impl ImportOfferDetailsCommand {
    pub fn guard(&self) -> Result<(), String> {
        if let Some(value) = &self.availability {
            value.guard()?;
        }
        if let Some(value) = &self.item_condition {
            value.guard()?;
        }
        super::event::guard_was_price(&self.price.current, self.price.previous.as_ref())?;
        if self.content.description.contains('\0') {
            return Err(err_invalid_description());
        }
        Ok(())
    }
}

fn err_invalid_description() -> String {
    "description must not contain NUL".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_import_contract_round_trips() {
        let payload: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/import_offer_details.json"
        ))
        .unwrap();
        let command: ImportOfferDetailsCommand = serde_json::from_value(payload.clone()).unwrap();
        command.guard().unwrap();
        assert_eq!(command.content.title, "Charger");
        assert_eq!(command.identifiers.sku.as_deref(), Some("A1"));
        assert_eq!(command.price.current.amount(), 1999);
        assert_eq!(serde_json::to_value(command).unwrap(), payload);
    }

    #[test]
    fn old_flat_import_contract_is_rejected() {
        let payload = serde_json::json!({"retailer_code":"AnkerAu", "title":"Charger", "price":{"amount":1999,"currency":"AUD"}, "url":"https://example.com/charger"});
        assert!(serde_json::from_value::<ImportOfferDetailsCommand>(payload).is_err());
    }
}
