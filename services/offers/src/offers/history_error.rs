use super::{event::RetailerOfferEvent, retailer_offer_id::RetailerOfferId};
use testimony::prelude::EventEnvelope;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum OfferHistoryError {
    #[error("offer history is empty")]
    EmptyHistory,
    #[error("expected {expected} at sequence 1, got {actual}")]
    UnexpectedFirstEvent {
        expected: &'static str,
        actual: String,
    },
    #[error("expected sequence {expected}, got {actual}")]
    InvalidSequence { expected: u64, actual: u64 },
    #[error("ID mismatch at sequence {sequence}: expected {expected}, got {actual}")]
    IdMismatch {
        sequence: u64,
        expected: String,
        actual: String,
    },
    #[error("duplicate creation at sequence {sequence}")]
    DuplicateCreation { sequence: u64 },
    #[error("sequence exhausted after {sequence}")]
    SequenceOverflow { sequence: u64 },
    #[error("invalid event data at sequence {sequence}: {message}")]
    InvalidEventData { sequence: u64, message: String },
}

impl OfferHistoryError {
    pub(super) fn empty() -> Self {
        Self::EmptyHistory
    }
    pub(super) fn first(expected: &'static str, actual: &RetailerOfferEvent) -> Self {
        Self::UnexpectedFirstEvent {
            expected,
            actual: event_name(actual),
        }
    }
    pub(super) fn duplicate(sequence: u64) -> Self {
        Self::DuplicateCreation { sequence }
    }
    pub(super) fn invalid(sequence: u64, message: impl ToString) -> Self {
        Self::InvalidEventData {
            sequence,
            message: message.to_string(),
        }
    }
    pub(super) fn next(sequence: u64) -> Result<u64, Self> {
        sequence
            .checked_add(1)
            .ok_or(Self::SequenceOverflow { sequence })
    }
    pub(super) fn guard_envelope(
        id: &RetailerOfferId,
        expected: u64,
        envelope: &EventEnvelope<RetailerOfferEvent>,
    ) -> Result<(), Self> {
        let sequence = envelope.sequence();
        if sequence != expected {
            return Err(Self::InvalidSequence {
                expected,
                actual: sequence,
            });
        }
        let actual = envelope.aggregate_id();
        if actual != id.value() {
            return Err(Self::IdMismatch {
                sequence,
                expected: id.to_string(),
                actual,
            });
        }
        Ok(())
    }
}

fn event_name(event: &RetailerOfferEvent) -> String {
    match event {
        RetailerOfferEvent::OfferAvailabilityChanged(_) => "OfferAvailabilityChangedNextGeneration",
        RetailerOfferEvent::OfferItemConditionChanged(_) => {
            "OfferItemConditionChangedNextGeneration"
        }
        RetailerOfferEvent::OfferPriceChanged(_) => "OfferPriceChangedNextGeneration",
        RetailerOfferEvent::OfferPriceConfirmed(_) => "OfferPriceConfirmed",
        RetailerOfferEvent::OfferHistoricPriceAdded(_) => "OfferHistoricPriceAdded",
        RetailerOfferEvent::OfferDescriptionChanged(_) => "OfferDescriptionChanged",
        RetailerOfferEvent::OfferImageRemoved(_) => "OfferImageRemoved",
        RetailerOfferEvent::OfferLanguageChanged(_) => "OfferLanguageChanged",
        RetailerOfferEvent::OfferTargetMarketChanged(_) => "OfferTargetMarketChanged",
        RetailerOfferEvent::OfferGtinChanged(_) => "OfferGtinChanged",
        RetailerOfferEvent::OfferSkuChanged(_) => "OfferSkuChanged",
        RetailerOfferEvent::OfferShopifyProductGidChanged(_) => "OfferShopifyProductGidChanged",
        RetailerOfferEvent::OfferShopifyProductVariantGidChanged(_) => {
            "OfferShopifyProductVariantGidChanged"
        }
        RetailerOfferEvent::OfferCreated(_) => "OfferCreatedNextGeneration",
        RetailerOfferEvent::OfferImageAdded(_) => "OfferImageAdded",
        RetailerOfferEvent::OfferTitleChanged(_) => "OfferTitleChanged",
    }
    .to_owned()
}
