use super::tests::{assert_price_confirmation, create, input};
use super::*;
use crate::infra::records::OfferEventRecord;
use everyday::prelude::CountryCode;
use kernel::ApplicationEvent;
use testimony::prelude::HasAggregateId;

fn update(
    aggregate: &mut RetailerOfferAggregate,
    identifiers: ExternalOfferIdentifiers,
    target_market: TargetMarket,
) {
    let request = input();
    aggregate.update(
        identifiers,
        target_market,
        request.language,
        request.content,
        request.price,
        request.availability,
        request.item_condition,
        Utc::now(),
    );
}

#[test]
fn market_changes_emit_one_event_and_equal_markets_emit_nothing() {
    let eu = input().target_market;
    let hr = TargetMarket::Country {
        code: CountryCode::HR,
    };
    let au = TargetMarket::Country {
        code: CountryCode::AU,
    };
    for (previous, next) in [(eu, eu), (hr, hr), (hr, au), (hr, eu), (eu, hr)] {
        let mut request = input();
        request.target_market = previous;
        let original = request.clone();
        let mut aggregate = create(request);
        let created_at = aggregate.state.created_at;
        let before = Utc::now();
        update(&mut aggregate, original.identifiers.clone(), next);
        assert_eq!(aggregate.state.target_market, next);
        assert_eq!(aggregate.state.identifiers, original.identifiers);
        assert_price_confirmation(&aggregate);
        assert_eq!(aggregate.state.created_at, created_at);
        assert_eq!(aggregate.state.id, original.id);
        assert_eq!(aggregate.state.retailer_code(), &original.retailer_code);
        assert_eq!(aggregate.state.url(), original.url);
        assert_eq!(aggregate.state.language, original.language);
        assert_eq!(aggregate.state.content, original.content);
        assert_eq!(aggregate.state.price.last, original.price);
        assert_eq!(
            aggregate.state.price.history,
            std::slice::from_ref(&original.price)
        );
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
        assert!(aggregate.state.updated_at >= before);
        assert!(aggregate.state.updated_at <= Utc::now());
        let envelope = &aggregate.uncommitted_events[1];
        assert_eq!(envelope.sequence(), 2);
        let event = envelope.event();
        assert_eq!(event.event_type(), "OfferTargetMarketChanged");
        assert_eq!(event.aggregate_id(), "offer-next");
        let payload = serde_json::to_value(event).unwrap();
        let expected = serde_json::json!({
            "id": "offer-next",
            "previous_target_market": previous,
            "target_market": next,
            "updated_at": aggregate.state.updated_at,
        });
        assert_eq!(payload, expected);
        let decoded: RetailerOfferEvent = serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(decoded.event_type(), "OfferTargetMarketChanged");
        assert_eq!(serde_json::to_value(decoded).unwrap(), expected);

        let record = OfferEventRecord::from(event);
        assert_eq!(record.aggregate_id(), "offer-next");
        let stored = serde_json::to_value(record).unwrap();
        assert_eq!(stored["type"], "OfferTargetMarketChanged.v1");
        assert_eq!(stored["data"], expected);
        let decoded: OfferEventRecord = serde_json::from_value(stored).unwrap();
        let restored = RetailerOfferEvent::from(decoded);
        assert_eq!(restored.event_type(), "OfferTargetMarketChanged");
        assert_eq!(serde_json::to_value(restored).unwrap(), expected);

        let updated_at = aggregate.state.updated_at;
        update(&mut aggregate, original.identifiers, next);
        assert_price_confirmation(&aggregate);
        assert_eq!(aggregate.sequence, 4);
        assert_eq!(aggregate.uncommitted_events.len(), 4);
        assert!(aggregate.state.updated_at >= updated_at);
    }
}

#[test]
fn target_market_change_follows_identifier_changes() {
    let mut aggregate = create(input());
    let mut identifiers = aggregate.state.identifiers.clone();
    identifiers.gtin = None;
    let target_market = TargetMarket::Country {
        code: CountryCode::HR,
    };
    update(&mut aggregate, identifiers.clone(), target_market);
    assert_eq!(aggregate.state.identifiers, identifiers);
    assert_eq!(aggregate.state.target_market, target_market);
    assert_price_confirmation(&aggregate);
    assert_eq!(aggregate.sequence, 4);
    assert_eq!(aggregate.uncommitted_events.len(), 4);
    for (index, expected_type) in [
        "OfferGtinChanged",
        "OfferTargetMarketChanged",
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
fn sequence_overflow_preserves_both_identifiers_and_market() {
    let mut aggregate = create(input());
    aggregate.sequence = u64::MAX - 1;
    let original_identifiers = aggregate.state.identifiers.clone();
    let original_market = aggregate.state.target_market;
    let updated_at = aggregate.state.updated_at;
    let mut identifiers = original_identifiers.clone();
    identifiers.gtin = None;
    let target_market = TargetMarket::Country {
        code: CountryCode::HR,
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        update(&mut aggregate, identifiers, target_market);
    }));
    assert!(result.is_err());
    assert_eq!(aggregate.state.identifiers, original_identifiers);
    assert_eq!(aggregate.state.target_market, original_market);
    assert_eq!(aggregate.state.updated_at, updated_at);
    assert_eq!(aggregate.sequence, u64::MAX - 1);
    assert_eq!(aggregate.uncommitted_events.len(), 1);
}
