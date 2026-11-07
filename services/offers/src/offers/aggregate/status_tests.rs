use super::identifier_tests::assert_round_trip;
use super::tests::{create, input};
use super::*;
use kernel::ApplicationEvent;

fn update(
    aggregate: &mut RetailerOfferAggregate,
    availability: Option<Availability>,
    item_condition: Option<ItemCondition>,
) {
    let state = &aggregate.state;
    let identifiers = state.identifiers.clone();
    let target_market = state.target_market;
    let language = state.language.clone();
    let content = state.content.clone();
    let price = state.price.last.clone();
    aggregate.update(
        identifiers,
        target_market,
        language,
        content,
        price,
        availability,
        item_condition,
        Utc::now(),
    );
}

#[test]
fn availability_transitions_emit_distinct_events_and_update_state() {
    let values = [
        None,
        Some(Availability::InStock),
        Some(Availability::OutOfStock),
        Some(Availability::Other(" custom ".into())),
        Some(Availability::Other("custom".into())),
    ];
    for previous in &values {
        for next in &values {
            let mut original = input();
            original.availability = previous.clone();
            let mut aggregate = create(original.clone());
            let created_at = aggregate.state.created_at;
            let before = Utc::now();
            update(
                &mut aggregate,
                next.clone(),
                original.item_condition.clone(),
            );
            assert_eq!(&aggregate.state.availability, next);
            assert_eq!(aggregate.state.item_condition, original.item_condition);
            assert_eq!(aggregate.state.created_at, created_at);
            assert_eq!(aggregate.state.identifiers, original.identifiers);
            assert_eq!(aggregate.state.content, original.content);
            assert_eq!(aggregate.state.price.last, original.price);
            assert_eq!(aggregate.state.price.history.len(), 1);
            assert!(
                aggregate.state.updated_at >= before && aggregate.state.updated_at <= Utc::now()
            );
            assert_eq!(
                aggregate.uncommitted_events[1].event().event_type(),
                "OfferPriceConfirmed"
            );
            if previous == next {
                assert_eq!(aggregate.sequence, 2);
                assert_eq!(aggregate.uncommitted_events.len(), 2);
                continue;
            }
            assert_eq!(aggregate.sequence, 3);
            assert_eq!(aggregate.uncommitted_events.len(), 3);
            let envelope = &aggregate.uncommitted_events[2];
            assert_eq!(envelope.sequence(), 3);
            if let RetailerOfferEvent::OfferAvailabilityChanged(event) = envelope.event() {
                assert_eq!(&event.previous_availability, previous);
                assert_eq!(&event.availability, next);
                assert_eq!(event.updated_at, aggregate.state.updated_at);
            } else {
                panic!("unexpected transition event");
            }
            assert_round_trip(envelope.event());
            update(
                &mut aggregate,
                next.clone(),
                original.item_condition.clone(),
            );
            assert_eq!(aggregate.sequence, 4);
            assert_eq!(aggregate.uncommitted_events.len(), 4);
            super::tests::assert_price_confirmation(&aggregate);
        }
    }
}

#[test]
fn item_condition_transitions_emit_distinct_events_and_update_state() {
    let values = [
        None,
        Some(ItemCondition::New),
        Some(ItemCondition::Other(" used ".into())),
        Some(ItemCondition::Other("used".into())),
    ];
    for previous in &values {
        for next in &values {
            let mut original = input();
            original.item_condition = previous.clone();
            let mut aggregate = create(original.clone());
            let created_at = aggregate.state.created_at;
            let before = Utc::now();
            update(&mut aggregate, original.availability.clone(), next.clone());
            assert_eq!(&aggregate.state.item_condition, next);
            assert_eq!(aggregate.state.availability, original.availability);
            assert_eq!(aggregate.state.created_at, created_at);
            assert_eq!(aggregate.state.identifiers, original.identifiers);
            assert_eq!(aggregate.state.content, original.content);
            assert_eq!(aggregate.state.price.last, original.price);
            assert_eq!(aggregate.state.price.history.len(), 1);
            assert!(
                aggregate.state.updated_at >= before && aggregate.state.updated_at <= Utc::now()
            );
            assert_eq!(
                aggregate.uncommitted_events[1].event().event_type(),
                "OfferPriceConfirmed"
            );
            if previous == next {
                assert_eq!(aggregate.sequence, 2);
                assert_eq!(aggregate.uncommitted_events.len(), 2);
                continue;
            }
            assert_eq!(aggregate.sequence, 3);
            assert_eq!(aggregate.uncommitted_events.len(), 3);
            let envelope = &aggregate.uncommitted_events[2];
            assert_eq!(envelope.sequence(), 3);
            if let RetailerOfferEvent::OfferItemConditionChanged(event) = envelope.event() {
                assert_eq!(&event.previous_item_condition, previous);
                assert_eq!(&event.item_condition, next);
                assert_eq!(event.updated_at, aggregate.state.updated_at);
            } else {
                panic!("unexpected transition event");
            }
            assert_round_trip(envelope.event());
            update(&mut aggregate, original.availability.clone(), next.clone());
            assert_eq!(aggregate.sequence, 4);
            assert_eq!(aggregate.uncommitted_events.len(), 4);
            super::tests::assert_price_confirmation(&aggregate);
        }
    }
}

#[test]
fn both_fields_follow_pricing_with_contiguous_sequences() {
    let mut original = input();
    original.availability = None;
    let mut aggregate = create(original);
    update(&mut aggregate, Some(Availability::InStock), None);
    assert_eq!(aggregate.state.availability, Some(Availability::InStock));
    assert_eq!(aggregate.state.item_condition, None);
    assert_eq!(aggregate.sequence, 4);
    assert_eq!(aggregate.uncommitted_events.len(), 4);
    for (envelope, (sequence, expected)) in aggregate.uncommitted_events[1..].iter().zip([
        (2, "OfferPriceConfirmed"),
        (3, "OfferAvailabilityChangedNextGeneration"),
        (4, "OfferItemConditionChangedNextGeneration"),
    ]) {
        assert_eq!(envelope.sequence(), sequence);
        assert_eq!(envelope.event().event_type(), expected);
        assert_round_trip(envelope.event());
    }
}

#[test]
fn overflow_leaves_both_fields_and_events_unchanged() {
    let original = input();
    let mut aggregate = create(original.clone());
    aggregate.sequence = u64::MAX - 2;
    let updated_at = aggregate.state.updated_at;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        update(&mut aggregate, None, None);
    }));
    assert!(result.is_err());
    assert_eq!(aggregate.sequence, u64::MAX - 2);
    assert_eq!(aggregate.uncommitted_events.len(), 1);
    assert_eq!(aggregate.state.availability, original.availability);
    assert_eq!(aggregate.state.item_condition, original.item_condition);
    assert_eq!(aggregate.state.price.last, original.price);
    assert!(aggregate.state.price.history.is_empty());
    assert_eq!(aggregate.state.updated_at, updated_at);
}

#[test]
fn legacy_status_events_are_rejected() {
    for (field, value, _expected_type) in [
        (
            "availability",
            serde_json::json!("in_stock"),
            "OfferAvailabilityChanged",
        ),
        (
            "availability",
            serde_json::Value::Null,
            "OfferAvailabilityChanged",
        ),
        (
            "item_condition",
            serde_json::json!("new"),
            "OfferItemConditionChanged",
        ),
        (
            "item_condition",
            serde_json::Value::Null,
            "OfferItemConditionChanged",
        ),
    ] {
        let payload = serde_json::json!({"id": "offer-next", field: value});
        assert!(serde_json::from_value::<RetailerOfferEvent>(payload).is_err());
    }
}
