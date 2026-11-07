use ::inbox::prelude as inbox;
use ::inbox::prelude::poem::{InboxAdapter, InboxSubmission, SubmitInboxRequest};
use poem::{Error, Result, http::StatusCode};
use poem_openapi::{OpenApi, payload::Json};

use crate::offers::command::{CREATE_FROM_OFFER_DETAILS_COMMAND, ImportOfferDetailsCommand};

const IMPORT_SOURCE: &str = "offers-api";

pub(crate) use self::model::ImportOfferRequest;

mod model {
    use crate::offers::state::{Content, ExternalOfferIdentifiers, Price};
    use crate::offers::target_market::TargetMarket;
    use poem_openapi::{Object, types::Any};

    #[derive(Object)]
    pub struct ImportOfferRequest {
        pub retailer_code: String,
        pub url: String,
        pub identifiers: Any<ExternalOfferIdentifiers>,
        pub target_market: Any<TargetMarket>,
        pub language: String,
        pub content: Any<Content>,
        pub price: Any<Price>,
        pub availability: Option<Any<retailer_sauce::availability::Availability>>,
        pub item_condition: Option<Any<retailer_sauce::item_condition::ItemCondition>>,
        pub created_at: String,
    }
}

impl TryFrom<ImportOfferRequest> for inbox::SubmitInboxRequest {
    type Error = String;

    fn try_from(request: ImportOfferRequest) -> std::result::Result<Self, Self::Error> {
        let command = ImportOfferDetailsCommand {
            retailer_code: serde_json::from_value(serde_json::Value::String(
                request.retailer_code.clone(),
            ))
            .or_else(|_| bodul_shared::retailer::RetailerCode::from_str(&request.retailer_code))?,
            url: request.url,
            identifiers: request.identifiers.0,
            target_market: request.target_market.0,
            language: request.language,
            content: request.content.0,
            price: request.price.0,
            availability: request.availability.map(|value| value.0),
            item_condition: request.item_condition.map(|value| value.0),
            created_at: request.created_at.parse().map_err(err_invalid_created_at)?,
        };
        command.guard()?;
        let payload = serde_json::to_value(command).map_err(err_invalid_payload)?;
        Ok(Self {
            payload: Some(payload),
            routing_key: CREATE_FROM_OFFER_DETAILS_COMMAND.into(),
            message_id: None,
            correlation_id: None,
            source: Some(IMPORT_SOURCE.into()),
        })
    }
}

fn err_invalid_created_at(error: chrono::ParseError) -> String {
    format!("invalid created_at: {error}")
}

fn err_invalid_payload(error: serde_json::Error) -> String {
    format!("invalid import payload: {error}")
}

pub struct OffersApi {
    inbox: InboxAdapter,
}

impl OffersApi {
    pub fn new(inbox: InboxAdapter) -> Self {
        Self { inbox }
    }
}

#[OpenApi]
impl OffersApi {
    /// Validate an import and store it in the inbox as an `ImportOfferDetails`
    /// message; `/inbox/run` and `/commands/run` then create the offer.
    #[oai(path = "/tmp-import", method = "post")]
    async fn tmp_import(&self, request: Json<ImportOfferRequest>) -> Result<InboxSubmission> {
        let submission =
            inbox::SubmitInboxRequest::try_from(request.0).map_err(err_invalid_import_request)?;
        self.inbox
            .submit(Json(SubmitInboxRequest(submission)))
            .await
    }
}

fn err_invalid_import_request(message: String) -> Error {
    Error::from_string(message, StatusCode::BAD_REQUEST)
}

#[cfg(test)]
mod tests {
    use super::*;
    use poem_openapi::types::ParseFromJSON;

    #[test]
    fn request_submits_the_new_command_contract() {
        let payload: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/import_offer_details.json"
        ))
        .unwrap();
        let request = ImportOfferRequest::parse_from_json(Some(payload.clone()))
            .map_err(|error| error.into_message())
            .unwrap();
        let submission = inbox::SubmitInboxRequest::try_from(request).unwrap();
        assert_eq!(submission.routing_key, CREATE_FROM_OFFER_DETAILS_COMMAND);
        assert_eq!(submission.payload.unwrap(), payload);
    }
}
