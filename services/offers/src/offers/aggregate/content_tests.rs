use super::identifier_tests::assert_round_trip;
use super::tests::{assert_price_confirmation, create, input};
use super::*;
use everyday::prelude::CountryCode;
use kernel::ApplicationEvent;

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
fn title_and_description_changes_are_independent_and_preserve_exact_text() {
    for (title, description, expected_types) in [
        (None, None, vec![]),
        (Some(""), None, vec!["OfferTitleChanged"]),
        (None, Some(""), vec!["OfferDescriptionChanged"]),
        (
            Some(" New title "),
            Some(" <p>New description</p> "),
            vec!["OfferTitleChanged", "OfferDescriptionChanged"],
        ),
    ] {
        let original = input();
        let mut aggregate = create(original.clone());
        let created_at = aggregate.state.created_at;
        let mut request = original.clone();
        if let Some(title) = title {
            request.content.title = title.into();
        }
        if let Some(description) = description {
            request.content.description = description.into();
        }
        let before = Utc::now();
        update(&mut aggregate, request.clone());
        assert_price_confirmation(&aggregate);
        assert_eq!(aggregate.state.content, request.content);
        assert_eq!(aggregate.state.identifiers, original.identifiers);
        assert_eq!(aggregate.state.target_market, original.target_market);
        assert_eq!(aggregate.state.language, original.language);
        assert_eq!(aggregate.state.price.last, original.price);
        assert_eq!(aggregate.state.availability, original.availability);
        assert_eq!(aggregate.state.item_condition, original.item_condition);
        assert_eq!(aggregate.state.created_at, created_at);
        assert_eq!(aggregate.sequence, 2 + expected_types.len() as u64);
        assert_eq!(aggregate.uncommitted_events.len(), 2 + expected_types.len());
        for (index, expected_type) in expected_types.iter().enumerate() {
            let envelope = &aggregate.uncommitted_events[index + 1];
            assert_eq!(envelope.sequence(), index as u64 + 2);
            assert_eq!(envelope.event().event_type(), *expected_type);
            match envelope.event() {
                RetailerOfferEvent::OfferTitleChanged(event) => {
                    assert_eq!(event.title, request.content.title);
                }
                RetailerOfferEvent::OfferDescriptionChanged(event) => {
                    assert_eq!(event.previous_description, original.content.description);
                    assert_eq!(event.description, request.content.description);
                    assert_eq!(event.updated_at, aggregate.state.updated_at);
                }
                _ => panic!("unexpected content event"),
            }
            assert_round_trip(envelope.event());
        }
        assert!(aggregate.state.updated_at >= before && aggregate.state.updated_at <= Utc::now());
        let updated_at = aggregate.state.updated_at;
        let sequence = aggregate.sequence;
        update(&mut aggregate, request);
        assert_price_confirmation(&aggregate);
        assert_eq!(aggregate.sequence, sequence + 1);
        assert_eq!(aggregate.uncommitted_events.len() as u64, sequence + 1);
        assert!(aggregate.state.updated_at >= updated_at);
    }
}

#[test]
fn images_add_once_then_remove_missing_urls_without_filtering() {
    let mut original = input();
    original.content.images = ["a", "b", "a", "/existing"].map(String::from).to_vec();
    let mut aggregate = create(original.clone());
    let mut request = original.clone();
    request.content.images = ["b", "c", "c", "/existing", "", "ftp://example.com/new"]
        .map(String::from)
        .to_vec();
    update(&mut aggregate, request.clone());
    assert_eq!(
        aggregate.state.content.images,
        ["b", "/existing", "c", "", "ftp://example.com/new"]
    );
    assert_price_confirmation(&aggregate);
    assert_eq!(aggregate.sequence, 6);
    assert_eq!(aggregate.uncommitted_events.len(), 6);
    let mut replayed_images = original.content.images;
    for (index, envelope) in aggregate.uncommitted_events[1..5].iter().enumerate() {
        assert_eq!(envelope.sequence(), index as u64 + 2);
        match envelope.event() {
            RetailerOfferEvent::OfferImageAdded(event) => {
                assert!(index < 3);
                assert_eq!(event.position as usize, replayed_images.len());
                replayed_images.push(event.image_url.clone());
            }
            RetailerOfferEvent::OfferImageRemoved(event) => {
                assert_eq!(index, 3);
                assert_eq!(event.image_url, "a");
                assert_eq!(event.updated_at, aggregate.state.updated_at);
                replayed_images.retain(|image| image != &event.image_url);
            }
            _ => panic!("unexpected image event"),
        }
        assert_round_trip(envelope.event());
    }
    assert_eq!(aggregate.state.content.images, replayed_images);
    let updated_at = aggregate.state.updated_at;
    update(&mut aggregate, request);
    assert_price_confirmation(&aggregate);
    assert_eq!(aggregate.sequence, 7);
    assert_eq!(aggregate.uncommitted_events.len(), 7);
    assert!(aggregate.state.updated_at >= updated_at);
}

#[test]
fn empty_image_list_removes_all_missing_urls_once() {
    let mut original = input();
    original.content.images = ["a", "a", "b"].map(String::from).to_vec();
    let mut aggregate = create(original.clone());
    let mut request = original;
    request.content.images.clear();
    update(&mut aggregate, request.clone());
    assert!(aggregate.state.content.images.is_empty());
    assert_price_confirmation(&aggregate);
    assert_eq!(aggregate.sequence, 4);
    assert_eq!(aggregate.uncommitted_events.len(), 4);
    for (envelope, image) in aggregate.uncommitted_events[1..].iter().zip(["a", "b"]) {
        let RetailerOfferEvent::OfferImageRemoved(event) = envelope.event() else {
            panic!("expected image removal");
        };
        assert_eq!(event.image_url, image);
        assert_round_trip(envelope.event());
    }
    let updated_at = aggregate.state.updated_at;
    update(&mut aggregate, request);
    assert_price_confirmation(&aggregate);
    assert_eq!(aggregate.sequence, 5);
    assert!(aggregate.state.updated_at >= updated_at);
}

#[test]
fn image_reordering_and_repeated_existing_urls_only_confirm_the_price() {
    let mut original = input();
    original.content.images = ["a", "a", "b"].map(String::from).to_vec();
    let mut aggregate = create(original.clone());
    let mut request = original.clone();
    request.content.images = ["b", "a", "a", "a"].map(String::from).to_vec();
    update(&mut aggregate, request);
    assert_eq!(aggregate.state.content.images, original.content.images);
    assert_price_confirmation(&aggregate);
    assert_eq!(aggregate.sequence, 2);
    assert_eq!(aggregate.uncommitted_events.len(), 2);
    assert!(aggregate.state.updated_at >= aggregate.state.created_at);
}

fn combined_changes() -> OfferCreatedEvent {
    let mut request = input();
    request.identifiers.gtin = None;
    request.target_market = TargetMarket::Country {
        code: CountryCode::HR,
    };
    request.language = "hr".into();
    request.content.title = "New title".into();
    request.content.description = "New description".into();
    request.content.images = vec!["/new-image".into()];
    request
}

#[test]
fn content_events_follow_identifiers_market_and_language() {
    let mut aggregate = create(input());
    let request = combined_changes();
    update(&mut aggregate, request.clone());
    assert_eq!(aggregate.state.identifiers, request.identifiers);
    assert_eq!(aggregate.state.target_market, request.target_market);
    assert_eq!(aggregate.state.language, request.language);
    assert_eq!(aggregate.state.content, request.content);
    let expected_types = [
        "OfferGtinChanged",
        "OfferTargetMarketChanged",
        "OfferLanguageChanged",
        "OfferTitleChanged",
        "OfferDescriptionChanged",
        "OfferImageAdded",
        "OfferImageRemoved",
        "OfferPriceConfirmed",
    ];
    assert_eq!(aggregate.sequence, 9);
    assert_eq!(aggregate.uncommitted_events.len(), 9);
    for (index, expected_type) in expected_types.into_iter().enumerate() {
        let envelope = &aggregate.uncommitted_events[index + 1];
        assert_eq!(envelope.sequence(), index as u64 + 2);
        assert_eq!(envelope.event().event_type(), expected_type);
        assert_round_trip(envelope.event());
    }
}

#[test]
fn sequence_overflow_does_not_partially_change_content_or_other_facts() {
    let original = input();
    let mut aggregate = create(original.clone());
    aggregate.sequence = u64::MAX - 6;
    let updated_at = aggregate.state.updated_at;
    let request = combined_changes();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        update(&mut aggregate, request)
    }));
    assert!(result.is_err());
    assert_eq!(aggregate.state.identifiers, original.identifiers);
    assert_eq!(aggregate.state.target_market, original.target_market);
    assert_eq!(aggregate.state.language, original.language);
    assert_eq!(aggregate.state.content, original.content);
    assert_eq!(aggregate.state.updated_at, updated_at);
    assert_eq!(aggregate.sequence, u64::MAX - 6);
    assert_eq!(aggregate.uncommitted_events.len(), 1);
}
