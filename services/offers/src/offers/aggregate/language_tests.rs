use super::tests::{assert_price_confirmation, create, input};
use super::*;
use crate::infra::records::OfferEventRecord;
use everyday::prelude::CountryCode;
use kernel::ApplicationEvent;
use testimony::prelude::HasAggregateId;

fn update(aggregate: &mut RetailerOfferAggregate, request: OfferCreatedEvent) {
    aggregate.update(
        request.identifiers,
        request.target_market,
        request.language,
        request.content,
        request.price,
        request.availability,
        request.item_condition,
        Utc::now(),
    );
}

#[test]
fn language_changes_preserve_exact_values_and_equal_values_emit_nothing() {
    for (previous, next) in [
        ("en", "en"),
        ("", ""),
        ("en", "hr"),
        ("en", "EN"),
        ("en", " en "),
        ("en", ""),
    ] {
        let mut original = input();
        original.language = previous.into();
        let mut aggregate = create(original.clone());
        let created_at = aggregate.state.created_at;
        let mut request = original.clone();
        request.language = next.into();
        let before = Utc::now();
        update(&mut aggregate, request.clone());
        assert_eq!(aggregate.state.language, next);
        assert_price_confirmation(&aggregate);
        assert_eq!(aggregate.state.created_at, created_at);
        assert_eq!(aggregate.state.id, original.id);
        assert_eq!(aggregate.state.retailer_code(), &original.retailer_code);
        assert_eq!(aggregate.state.url(), original.url);
        assert_eq!(aggregate.state.identifiers, original.identifiers);
        assert_eq!(aggregate.state.target_market, original.target_market);
        assert_eq!(aggregate.state.content, original.content);
        assert_eq!(aggregate.state.price.last, original.price);
        assert_eq!(aggregate.state.price.history, [original.price.clone()]);
        assert_eq!(aggregate.state.availability, original.availability);
        assert_eq!(aggregate.state.item_condition, original.item_condition);
        if previous == next {
            assert_eq!(aggregate.sequence, 2);
            assert_eq!(aggregate.uncommitted_events.len(), 2);
            assert!(aggregate.state.updated_at >= created_at);
            continue;
        }

        assert_eq!(aggregate.sequence, 3);
        assert_eq!(aggregate.uncommitted_events.len(), 3);
        assert!(aggregate.state.updated_at >= before && aggregate.state.updated_at <= Utc::now());
        let envelope = &aggregate.uncommitted_events[1];
        assert_eq!(envelope.sequence(), 2);
        let event = envelope.event();
        assert_eq!(event.event_type(), "OfferLanguageChanged");
        assert_eq!(event.aggregate_id(), "offer-next");
        let expected = serde_json::json!({
            "id": "offer-next",
            "previous_language": previous,
            "language": next,
            "updated_at": aggregate.state.updated_at,
        });
        assert_eq!(serde_json::to_value(event).unwrap(), expected);
        let decoded: RetailerOfferEvent = serde_json::from_value(expected.clone()).unwrap();
        assert_eq!(decoded.event_type(), "OfferLanguageChanged");
        assert_eq!(serde_json::to_value(decoded).unwrap(), expected);
        let record = OfferEventRecord::from(event);
        assert_eq!(record.aggregate_id(), "offer-next");
        let stored = serde_json::to_value(record).unwrap();
        assert_eq!(stored["type"], "OfferLanguageChanged.v1");
        assert_eq!(stored["data"], expected);
        let decoded: OfferEventRecord = serde_json::from_value(stored).unwrap();
        let restored = RetailerOfferEvent::from(decoded);
        assert_eq!(restored.event_type(), "OfferLanguageChanged");
        assert_eq!(serde_json::to_value(restored).unwrap(), expected);

        let updated_at = aggregate.state.updated_at;
        update(&mut aggregate, request);
        assert_price_confirmation(&aggregate);
        assert_eq!(aggregate.sequence, 4);
        assert_eq!(aggregate.uncommitted_events.len(), 4);
        assert!(aggregate.state.updated_at >= updated_at);
    }
}

#[test]
fn language_change_follows_identifier_and_market_changes() {
    let mut aggregate = create(input());
    let mut request = input();
    request.identifiers.gtin = None;
    request.target_market = TargetMarket::Country {
        code: CountryCode::HR,
    };
    request.language = "hr".into();
    update(&mut aggregate, request.clone());
    assert_eq!(aggregate.state.identifiers, request.identifiers);
    assert_eq!(aggregate.state.target_market, request.target_market);
    assert_eq!(aggregate.state.language, request.language);
    assert_price_confirmation(&aggregate);
    assert_eq!(aggregate.sequence, 5);
    assert_eq!(aggregate.uncommitted_events.len(), 5);
    for (index, expected_type) in [
        "OfferGtinChanged",
        "OfferTargetMarketChanged",
        "OfferLanguageChanged",
        "OfferPriceConfirmed",
    ]
    .into_iter()
    .enumerate()
    {
        let envelope = &aggregate.uncommitted_events[index + 1];
        assert_eq!(envelope.sequence(), index as u64 + 2);
        assert_eq!(envelope.event().event_type(), expected_type);
        let payload = serde_json::to_value(envelope.event()).unwrap();
        assert_eq!(
            payload["updated_at"],
            serde_json::json!(aggregate.state.updated_at)
        );
    }
}

#[test]
fn sequence_overflow_preserves_identifiers_market_and_language() {
    let original = input();
    let mut aggregate = create(original.clone());
    aggregate.sequence = u64::MAX - 2;
    let updated_at = aggregate.state.updated_at;
    let mut request = original.clone();
    request.identifiers.gtin = None;
    request.target_market = TargetMarket::Country {
        code: CountryCode::HR,
    };
    request.language = "hr".into();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        update(&mut aggregate, request);
    }));
    assert!(result.is_err());
    assert_eq!(aggregate.state.identifiers, original.identifiers);
    assert_eq!(aggregate.state.target_market, original.target_market);
    assert_eq!(aggregate.state.language, original.language);
    assert_eq!(aggregate.state.updated_at, updated_at);
    assert_eq!(aggregate.sequence, u64::MAX - 2);
    assert_eq!(aggregate.uncommitted_events.len(), 1);
}
