use super::*;
use crate::infra::records::OfferEventRecord;
use kernel::ApplicationEvent;
use money::{Currency, Money};
use retailer_sauce::localization::RegionCode;
use testimony::prelude::HasAggregateId;

pub(super) fn assert_price_confirmation(aggregate: &RetailerOfferAggregate) {
    let envelope = aggregate.uncommitted_events.last().unwrap();
    assert_eq!(envelope.sequence(), aggregate.sequence);
    let RetailerOfferEvent::OfferPriceConfirmed(event) = envelope.event() else {
        panic!("expected a price confirmation after the other changes");
    };
    assert_eq!(event.id, aggregate.state.id);
    assert_eq!(event.price, aggregate.state.price.last);
    assert_eq!(event.updated_at, aggregate.state.updated_at);
}

pub(super) fn input() -> OfferCreatedEvent {
    OfferCreatedEvent {
        id: RetailerOfferId::new("offer-next"),
        retailer_code: RetailerCode::AnkerCom,
        url: "https://example.com/charger".into(),
        identifiers: ExternalOfferIdentifiers {
            gtin: Some("1234567890123".into()),
            sku: Some("charger-1".into()),
            shopify_product_gid: Some("gid://shopify/Product/1".into()),
            shopify_product_variant_gid: Some("gid://shopify/ProductVariant/2".into()),
        },
        target_market: TargetMarket::Region {
            code: RegionCode::EuropeanUnion,
        },
        language: "en".into(),
        content: Content {
            title: " Charger ".into(),
            description: "<p>Fast charger</p>".into(),
            images: vec!["https://example.com/charger.jpg".into()],
        },
        price: Price {
            current: Money::new(1999, Currency::USD),
            previous: Some(Money::new(2999, Currency::USD)),
            scraped_at: "2026-10-30T12:00:00Z".parse().unwrap(),
        },
        availability: Some(Availability::InStock),
        item_condition: Some(ItemCondition::New),
        created_at: "2026-10-30T12:00:00Z".parse().unwrap(),
    }
}

pub(super) fn create(input: OfferCreatedEvent) -> RetailerOfferAggregate {
    RetailerOfferAggregate::create(
        input.id,
        input.retailer_code,
        input.url,
        input.identifiers,
        input.target_market,
        input.language,
        input.content,
        input.price,
        input.availability,
        input.item_condition,
        input.created_at,
    )
}

fn new_event(input: OfferCreatedEvent) -> OfferCreatedEvent {
    OfferCreatedEvent::new(
        input.id,
        input.retailer_code,
        input.url,
        input.identifiers,
        input.target_market,
        input.language,
        input.content,
        input.price,
        input.availability,
        input.item_condition,
        input.created_at,
    )
}

#[test]
fn event_constructor_preserves_valid_values() {
    let expected = input();
    let event = new_event(expected.clone());
    assert_eq!(event, expected);
}

#[test]
fn event_constructor_preserves_invalid_and_duplicate_images() {
    for image in ["/relative-image", "https://example.com/charger.jpg"] {
        let mut request = input();
        request.content.images.push(image.into());
        assert_eq!(new_event(request.clone()), request);
    }
}

#[test]
fn creation_records_matching_next_generation_state_and_event() {
    let mut expected = input();
    expected.content.images.extend([
        "https://example.com/charger.jpg".into(),
        "/invalid-relative-image".into(),
    ]);
    let request = expected.clone();
    let aggregate = create(request);
    let state = &aggregate.state;
    assert_eq!(state.created_at, expected.created_at);
    assert_eq!(state.updated_at, state.created_at);
    assert!(state.price.history.is_empty());
    assert_eq!(state.id, expected.id);
    assert_eq!(state.retailer_code(), &expected.retailer_code);
    assert_eq!(state.url(), expected.url);
    assert_eq!(state.identifiers, expected.identifiers);
    assert_eq!(state.target_market, expected.target_market);
    assert_eq!(state.language, expected.language);
    assert_eq!(state.content, expected.content);
    assert_eq!(state.price.last, expected.price);
    assert_eq!(state.availability, expected.availability);
    assert_eq!(state.item_condition, expected.item_condition);
    assert_eq!(aggregate.sequence, 1);
    assert_eq!(aggregate.uncommitted_events.len(), 1);
    let envelope = &aggregate.uncommitted_events[0];
    assert_eq!(envelope.sequence(), 1);
    let RetailerOfferEvent::OfferCreated(event) = envelope.event() else {
        panic!("expected next-generation creation");
    };
    assert_eq!(event, &expected);
}

#[test]
fn next_generation_creation_round_trips_with_its_own_event_and_record_types() {
    let aggregate = create(input());
    let event = aggregate.uncommitted_events[0].event();
    assert_eq!(event.event_type(), "OfferCreatedNextGeneration");
    assert_eq!(event.aggregate_id(), "offer-next");
    let payload = serde_json::to_value(event).unwrap();
    let decoded: RetailerOfferEvent = serde_json::from_value(payload.clone()).unwrap();
    assert!(matches!(decoded, RetailerOfferEvent::OfferCreated(_)));
    assert_eq!(serde_json::to_value(decoded).unwrap(), payload);

    let record = OfferEventRecord::from(event);
    assert_eq!(record.aggregate_id(), "offer-next");
    let stored = serde_json::to_value(record).unwrap();
    assert_eq!(stored["type"], "OfferCreatedNextGeneration.v1");
    assert_eq!(stored["data"], payload);
    let decoded: OfferEventRecord = serde_json::from_value(stored).unwrap();
    let restored = RetailerOfferEvent::from(decoded);
    assert_eq!(restored.event_type(), "OfferCreatedNextGeneration");
    assert_eq!(serde_json::to_value(restored).unwrap(), payload);
}

#[test]
fn country_market_is_preserved_and_legacy_region_records_are_rejected() {
    let mut request = input();
    request.target_market = TargetMarket::Country {
        code: everyday::prelude::CountryCode::HR,
    };
    let expected_market = request.target_market;
    let aggregate = create(request);
    assert_eq!(aggregate.state.target_market, expected_market);
    let event = aggregate.uncommitted_events[0].event();
    let mut payload = serde_json::to_value(event).unwrap();
    assert_eq!(
        payload["target_market"],
        serde_json::json!({"kind": "country", "code": "HR"})
    );
    payload["target_market"] = serde_json::json!({"kind": "region", "label": "European Union"});
    assert!(serde_json::from_value::<OfferCreatedEvent>(payload.clone()).is_err());
    let record = serde_json::json!({"type": "OfferCreatedNextGeneration.v1", "data": payload});
    assert!(serde_json::from_value::<OfferEventRecord>(record).is_err());
}

#[test]
fn creation_preserves_unvalidated_facts() {
    let mut invalid_inputs = Vec::new();
    let mut request = input();
    request.price.previous = Some(Money::new(1999, Currency::USD));
    invalid_inputs.push(request);
    let mut request = input();
    request.price.previous = Some(Money::new(2999, Currency::AUD));
    invalid_inputs.push(request);
    let mut request = input();
    request.content.title = " ".into();
    invalid_inputs.push(request);
    let mut request = input();
    request.language = "".into();
    invalid_inputs.push(request);
    let mut request = input();
    request.identifiers.shopify_product_variant_gid = Some("".into());
    invalid_inputs.push(request);
    let mut request = input();
    request.availability = Some(Availability::Other("".into()));
    invalid_inputs.push(request);
    let mut request = input();
    request.item_condition = Some(ItemCondition::Other("\0".into()));
    invalid_inputs.push(request);
    for request in invalid_inputs {
        assert_eq!(new_event(request.clone()), request);
        let aggregate = create(request.clone());
        let replayed = RetailerOfferAggregate::from_history(
            request.id.clone(),
            aggregate.uncommitted_events.clone(),
        )
        .unwrap();
        assert_eq!(replayed.state, aggregate.state);
        assert_eq!(replayed.sequence, 1);
        assert!(replayed.uncommitted_events.is_empty());

        let RetailerOfferEvent::OfferCreated(event) = aggregate.uncommitted_events[0].event()
        else {
            panic!("expected next-generation creation");
        };
        assert_eq!(event, &request);
    }
}

#[test]
fn replay_matches_updates_without_recording_events() {
    let original = input();
    let mut aggregate = create(original.clone());
    let mut changed = original.clone();
    changed.identifiers.gtin = None;
    changed.identifiers.sku = None;
    changed.identifiers.shopify_product_gid = None;
    changed.identifiers.shopify_product_variant_gid = None;
    changed.target_market = TargetMarket::Country {
        code: everyday::prelude::CountryCode::HR,
    };
    changed.language = "de".into();
    changed.content.title = "New title".into();
    changed.content.description = "New description".into();
    changed.content.images = vec!["https://example.com/new.jpg".into()];
    changed.availability = None;
    changed.item_condition = None;
    changed.price.current = Money::new(999, Currency::USD);
    for offset in [60, 120, -60] {
        changed.price.scraped_at = original.price.scraped_at + chrono::Duration::seconds(offset);
        aggregate.update(
            changed.identifiers.clone(),
            changed.target_market,
            changed.language.clone(),
            changed.content.clone(),
            changed.price.clone(),
            changed.availability.clone(),
            changed.item_condition.clone(),
            original.created_at + chrono::Duration::seconds(180),
        );
    }
    let mut replayed =
        RetailerOfferAggregate::from_history(original.id, aggregate.uncommitted_events.clone())
            .unwrap();
    assert_eq!(replayed.state, aggregate.state);
    assert_eq!(replayed.sequence, aggregate.sequence);
    assert!(replayed.uncommitted_events.is_empty());
    let sequence = replayed.sequence;
    replayed.update(
        changed.identifiers,
        changed.target_market,
        changed.language,
        changed.content,
        changed.price,
        changed.availability,
        changed.item_condition,
        original.created_at,
    );
    assert_eq!(replayed.uncommitted_events[0].sequence(), sequence + 1);
}

#[test]
fn apply_records_and_evolves_a_new_event() {
    let mut aggregate = create(input());
    let event = OfferTitleChangedEvent::new(aggregate.state.id.clone(), "Changed".into());
    aggregate.apply(RetailerOfferEvent::OfferTitleChanged(event));
    assert_eq!(aggregate.state.content.title, "Changed");
    assert_eq!(aggregate.sequence, 2);
    assert_eq!(aggregate.uncommitted_events.len(), 2);
    assert_eq!(aggregate.uncommitted_events[1].sequence(), 2);
}

#[test]
fn replay_reports_structural_errors() {
    let request = input();
    let id = request.id.clone();
    let created = RetailerOfferEvent::OfferCreated(request);
    let title =
        RetailerOfferEvent::OfferTitleChanged(OfferTitleChangedEvent::new(id.clone(), "".into()));
    let cases = [
        (vec![], OfferHistoryError::EmptyHistory),
        (
            vec![EventEnvelope::new(title.clone(), 1)],
            OfferHistoryError::first("OfferCreatedNextGeneration", &title),
        ),
        (
            vec![EventEnvelope::new(created.clone(), 0)],
            OfferHistoryError::InvalidSequence {
                expected: 1,
                actual: 0,
            },
        ),
        (
            vec![EventEnvelope::new(created.clone(), 2)],
            OfferHistoryError::InvalidSequence {
                expected: 1,
                actual: 2,
            },
        ),
        (
            vec![
                EventEnvelope::new(created.clone(), 1),
                EventEnvelope::new(created.clone(), 2),
            ],
            OfferHistoryError::DuplicateCreation { sequence: 2 },
        ),
        (
            vec![
                EventEnvelope::new(created.clone(), 1),
                EventEnvelope::new(title.clone(), 3),
            ],
            OfferHistoryError::InvalidSequence {
                expected: 2,
                actual: 3,
            },
        ),
        (
            vec![
                EventEnvelope::new(created.clone(), 1),
                EventEnvelope::new(title.clone(), 1),
            ],
            OfferHistoryError::InvalidSequence {
                expected: 2,
                actual: 1,
            },
        ),
        (
            vec![
                EventEnvelope::new(created.clone(), 1),
                EventEnvelope::new(title.clone(), 3),
                EventEnvelope::new(title, 2),
            ],
            OfferHistoryError::InvalidSequence {
                expected: 2,
                actual: 3,
            },
        ),
    ];
    for (history, expected) in cases {
        assert_eq!(
            RetailerOfferAggregate::from_history(id.clone(), history).err(),
            Some(expected)
        );
    }
    let history = vec![EventEnvelope::new(created.clone(), 1)];
    assert_eq!(
        RetailerOfferAggregate::from_history(RetailerOfferId::new("other"), history).err(),
        Some(OfferHistoryError::IdMismatch {
            sequence: 1,
            expected: "other".into(),
            actual: id.to_string()
        })
    );
    let wrong = RetailerOfferEvent::OfferTitleChanged(OfferTitleChangedEvent::new(
        RetailerOfferId::new("other"),
        "".into(),
    ));
    let history = vec![EventEnvelope::new(created, 1), EventEnvelope::new(wrong, 2)];
    assert_eq!(
        RetailerOfferAggregate::from_history(id.clone(), history).err(),
        Some(OfferHistoryError::IdMismatch {
            sequence: 2,
            expected: id.to_string(),
            actual: "other".into()
        })
    );
    assert_eq!(
        OfferHistoryError::next(u64::MAX),
        Err(OfferHistoryError::SequenceOverflow { sequence: u64::MAX })
    );
}

#[test]
fn replay_rejects_unsafe_image_positions() {
    let request = input();
    let id = request.id.clone();
    let created = EventEnvelope::new(RetailerOfferEvent::OfferCreated(request), 1);
    for position in [2, u32::MAX] {
        let image = OfferImageAddedEvent {
            id: id.clone(),
            image_url: "invalid".into(),
            position,
        };
        let history = vec![
            created.clone(),
            EventEnvelope::new(RetailerOfferEvent::OfferImageAdded(image), 2),
        ];
        assert!(matches!(
            RetailerOfferAggregate::from_history(id.clone(), history),
            Err(OfferHistoryError::InvalidEventData { sequence: 2, .. })
        ));
    }
    // Insertion at zero and at the end retains the handlers' unvalidated URL semantics.
    for position in [0, 1] {
        let image = OfferImageAddedEvent {
            id: id.clone(),
            image_url: "invalid".into(),
            position,
        };
        let history = vec![
            created.clone(),
            EventEnvelope::new(RetailerOfferEvent::OfferImageAdded(image), 2),
        ];
        let replay = RetailerOfferAggregate::from_history(id.clone(), history).unwrap();
        assert_eq!(replay.state.content.images[position as usize], "invalid");
        assert!(replay.uncommitted_events.is_empty());
    }
}
