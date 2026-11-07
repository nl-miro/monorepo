use super::identifier_tests::assert_round_trip;
use super::tests::{create, input};
use super::*;
use chrono::Duration;
use kernel::ApplicationEvent;
use money::{Currency, Money};

fn update(aggregate: &mut RetailerOfferAggregate, price: Price) {
    let identifiers = aggregate.state.identifiers.clone();
    let target_market = aggregate.state.target_market;
    let language = aggregate.state.language.clone();
    let content = aggregate.state.content.clone();
    let availability = aggregate.state.availability.clone();
    let item_condition = aggregate.state.item_condition.clone();
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
fn changed_and_confirmed_prices_replace_last_and_archive_previous() {
    let original = input();
    let cases = [
        (
            Money::new(1999, Currency::USD),
            Some(Money::new(2999, Currency::USD)),
            "OfferPriceConfirmed",
        ),
        (
            Money::new(2099, Currency::USD),
            Some(Money::new(2999, Currency::USD)),
            "OfferPriceChangedNextGeneration",
        ),
        (
            Money::new(1999, Currency::USD),
            Some(Money::new(3999, Currency::USD)),
            "OfferPriceChangedNextGeneration",
        ),
        (
            Money::new(2099, Currency::USD),
            Some(Money::new(3999, Currency::USD)),
            "OfferPriceChangedNextGeneration",
        ),
        (
            Money::new(1999, Currency::USD),
            None,
            "OfferPriceChangedNextGeneration",
        ),
        (
            Money::new(1999, Currency::AUD),
            Some(Money::new(2999, Currency::USD)),
            "OfferPriceChangedNextGeneration",
        ),
        (
            Money::new(1999, Currency::USD),
            Some(Money::new(2999, Currency::AUD)),
            "OfferPriceChangedNextGeneration",
        ),
    ];
    for seconds in [0, 60] {
        for (current, previous, expected_type) in &cases {
            let mut aggregate = create(original.clone());
            let created_at = aggregate.state.created_at;
            let price = Price {
                current: current.clone(),
                previous: previous.clone(),
                scraped_at: original.price.scraped_at + Duration::seconds(seconds),
            };
            let before = Utc::now();
            update(&mut aggregate, price.clone());
            assert_eq!(aggregate.sequence, 2);
            assert_eq!(aggregate.uncommitted_events.len(), 2);
            assert_eq!(aggregate.state.price.last, price);
            assert_eq!(
                aggregate.state.price.history,
                std::slice::from_ref(&original.price)
            );
            assert_eq!(aggregate.state.created_at, created_at);
            assert!(
                aggregate.state.updated_at >= before && aggregate.state.updated_at <= Utc::now()
            );
            let envelope = &aggregate.uncommitted_events[1];
            assert_eq!(envelope.sequence(), 2);
            assert_eq!(envelope.event().event_type(), *expected_type);
            match envelope.event() {
                RetailerOfferEvent::OfferPriceChanged(event) => {
                    assert_eq!(event.previous_price, original.price);
                    assert_eq!(event.price, price);
                    assert_eq!(event.updated_at, aggregate.state.updated_at);
                }
                RetailerOfferEvent::OfferPriceConfirmed(event) => {
                    assert_eq!(event.price, price);
                    assert_eq!(event.updated_at, aggregate.state.updated_at);
                }
                _ => panic!("expected a changed or confirmed price"),
            }
            assert_round_trip(envelope.event());
        }
    }
}

#[test]
fn previous_price_can_be_added_and_absence_can_be_confirmed() {
    let mut original = input();
    original.price.previous = None;
    for (previous, expected_type) in [
        (None, "OfferPriceConfirmed"),
        (
            Some(Money::new(2999, Currency::USD)),
            "OfferPriceChangedNextGeneration",
        ),
    ] {
        let mut aggregate = create(original.clone());
        let mut price = original.price.clone();
        price.previous = previous;
        price.scraped_at += Duration::seconds(1);
        update(&mut aggregate, price.clone());
        assert_eq!(aggregate.state.price.last, price);
        assert_eq!(
            aggregate.state.price.history,
            std::slice::from_ref(&original.price)
        );
        let event = aggregate.uncommitted_events[1].event();
        assert_eq!(event.event_type(), expected_type);
        assert_round_trip(event);
    }
}

#[test]
fn historic_prices_preserve_latest_even_when_amounts_match() {
    let original = input();
    let mut aggregate = create(original.clone());
    let mut changed = original.price.clone();
    changed.current = Money::new(2099, Currency::USD);
    changed.scraped_at += Duration::seconds(60);
    update(&mut aggregate, changed.clone());
    let mut confirmed = changed.clone();
    confirmed.scraped_at += Duration::seconds(60);
    update(&mut aggregate, confirmed.clone());

    let mut older_changed = original.price.clone();
    older_changed.current = Money::new(999, Currency::AUD);
    older_changed.previous = None;
    older_changed.scraped_at += Duration::seconds(30);
    let mut older_unchanged = confirmed.clone();
    older_unchanged.scraped_at -= Duration::seconds(30);
    for price in [&older_changed, &older_unchanged] {
        update(&mut aggregate, price.clone());
        assert_eq!(aggregate.state.price.last, confirmed);
        let envelope = aggregate.uncommitted_events.last().unwrap();
        assert_eq!(envelope.sequence(), aggregate.sequence);
        let RetailerOfferEvent::OfferHistoricPriceAdded(event) = envelope.event() else {
            panic!("an older scrape must produce a historic price event");
        };
        assert_eq!(&event.historic_price, price);
        assert_eq!(event.updated_at, aggregate.state.updated_at);
        assert_round_trip(envelope.event());
    }
    assert_eq!(
        aggregate.state.price.history,
        [original.price, changed, older_changed, older_unchanged]
    );
    assert_eq!(aggregate.sequence, 5);
    assert_eq!(aggregate.uncommitted_events.len(), 5);
}

#[test]
fn identical_observations_are_confirmed_each_time() {
    let original = input();
    let mut aggregate = create(original.clone());
    for count in 1..=2 {
        update(&mut aggregate, original.price.clone());
        super::tests::assert_price_confirmation(&aggregate);
        assert_eq!(aggregate.sequence, count as u64 + 1);
        assert_eq!(aggregate.uncommitted_events.len(), count + 1);
        assert_eq!(
            aggregate.state.price.history,
            vec![original.price.clone(); count]
        );
    }
}

#[test]
fn historic_pricing_does_not_discard_other_field_changes() {
    let original = input();
    let mut aggregate = create(original.clone());
    let mut request = original.clone();
    request.price.scraped_at -= Duration::seconds(1);
    request.language = "hr".into();
    aggregate.update(
        request.identifiers,
        request.target_market,
        request.language,
        request.content,
        request.price.clone(),
        request.availability,
        request.item_condition,
        Utc::now(),
    );
    assert_eq!(aggregate.state.language, "hr");
    assert_eq!(aggregate.state.price.last, original.price);
    assert_eq!(aggregate.state.price.history, [request.price]);
    assert_eq!(
        aggregate.uncommitted_events[1].event().event_type(),
        "OfferLanguageChanged"
    );
    assert_eq!(
        aggregate.uncommitted_events[2].event().event_type(),
        "OfferHistoricPriceAdded"
    );
    assert_eq!(aggregate.sequence, 3);
}

#[test]
fn sequence_overflow_does_not_change_last_or_history() {
    let original = input();
    let mut changed = original.price.clone();
    changed.current = Money::new(2099, Currency::USD);
    let mut historic = original.price.clone();
    historic.scraped_at -= Duration::seconds(1);
    for price in [original.price.clone(), changed, historic] {
        let mut aggregate = create(original.clone());
        aggregate.sequence = u64::MAX;
        let updated_at = aggregate.state.updated_at;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            update(&mut aggregate, price)
        }));
        assert!(result.is_err());
        assert_eq!(aggregate.sequence, u64::MAX);
        assert_eq!(aggregate.uncommitted_events.len(), 1);
        assert_eq!(aggregate.state.price.last, original.price);
        assert!(aggregate.state.price.history.is_empty());
        assert_eq!(aggregate.state.updated_at, updated_at);
    }
}

#[test]
fn legacy_price_events_are_rejected() {
    let cases = [
        (
            serde_json::json!({"id":"offer-next","price":{"amount":1999,"currency":"USD"}}),
            "OfferPriceChangedEvent",
            "OfferPriceChanged.v1",
        ),
        (
            serde_json::json!({"id":"offer-next","pricing":{"price":{"amount":1999,"currency":"USD"},"was_price":null}}),
            "OfferPricingChanged",
            "OfferPricingChanged.v1",
        ),
    ];
    for (payload, _expected_type, _expected_record) in cases {
        assert!(serde_json::from_value::<RetailerOfferEvent>(payload).is_err());
    }
}
