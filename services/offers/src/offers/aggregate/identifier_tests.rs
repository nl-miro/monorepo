use super::tests::{assert_price_confirmation, create, input};
use super::*;
use crate::infra::records::OfferEventRecord;
use kernel::ApplicationEvent;
use serde_json::json;
use testimony::prelude::HasAggregateId;

fn update(aggregate: &mut RetailerOfferAggregate, identifiers: ExternalOfferIdentifiers) {
    let request = input();
    aggregate.update(
        identifiers,
        request.target_market,
        request.language,
        request.content,
        request.price,
        request.availability,
        request.item_condition,
        Utc::now(),
    );
}

pub(super) fn assert_round_trip(event: &RetailerOfferEvent) {
    let payload = serde_json::to_value(event).unwrap();
    let decoded: RetailerOfferEvent = serde_json::from_value(payload.clone()).unwrap();
    assert_eq!(decoded.event_type(), event.event_type());
    assert_eq!(decoded.aggregate_id(), "offer-next");
    assert_eq!(serde_json::to_value(decoded).unwrap(), payload);

    let record = OfferEventRecord::from(event);
    assert_eq!(record.aggregate_id(), "offer-next");
    let stored = serde_json::to_value(record).unwrap();
    assert_eq!(stored["type"], format!("{}.v1", event.event_type()));
    assert_eq!(stored["data"], payload);
    let decoded: OfferEventRecord = serde_json::from_value(stored).unwrap();
    let restored = RetailerOfferEvent::from(decoded);
    assert_eq!(restored.event_type(), event.event_type());
    assert_eq!(serde_json::to_value(restored).unwrap(), payload);
}

#[test]
fn each_identifier_emits_changed_or_nothing() {
    type Field = (
        &'static str,
        &'static str,
        fn(&mut ExternalOfferIdentifiers) -> &mut Option<String>,
    );
    let fields: [Field; 4] = [
        ("gtin", "Gtin", |identifiers| &mut identifiers.gtin),
        ("sku", "Sku", |identifiers| &mut identifiers.sku),
        ("shopify_product_gid", "ShopifyProductGid", |identifiers| {
            &mut identifiers.shopify_product_gid
        }),
        (
            "shopify_product_variant_gid",
            "ShopifyProductVariantGid",
            |identifiers| &mut identifiers.shopify_product_variant_gid,
        ),
    ];
    for (field, name, select) in fields {
        for (previous, next, action) in [
            (None, None, None),
            (None, Some(""), Some("Changed")),
            (Some("old"), Some("old"), None),
            (Some("old"), Some(" new "), Some("Changed")),
            (Some("old"), None, Some("Changed")),
        ] {
            let mut request = input();
            *select(&mut request.identifiers) = previous.map(String::from);
            let mut aggregate = create(request);
            let previous_updated_at = aggregate.state.updated_at;
            let mut identifiers = aggregate.state.identifiers.clone();
            *select(&mut identifiers) = next.map(String::from);
            let expected = identifiers.clone();
            let before = Utc::now();
            update(&mut aggregate, identifiers);
            assert_price_confirmation(&aggregate);
            assert_eq!(aggregate.state.identifiers, expected);

            let Some(action) = action else {
                assert_eq!(aggregate.sequence, 2);
                assert_eq!(aggregate.uncommitted_events.len(), 2);
                assert!(aggregate.state.updated_at >= previous_updated_at);
                continue;
            };
            assert_eq!(aggregate.sequence, 3);
            assert_eq!(aggregate.uncommitted_events.len(), 3);
            assert!(aggregate.state.updated_at >= before);
            assert!(aggregate.state.updated_at <= Utc::now());
            let envelope = &aggregate.uncommitted_events[1];
            assert_eq!(envelope.sequence(), 2);
            let event = envelope.event();
            assert_eq!(event.event_type(), format!("Offer{name}{action}"));
            let mut payload = json!({"id": "offer-next", "updated_at": aggregate.state.updated_at});
            payload[format!("previous_{field}")] = json!(previous);
            payload[field] = json!(next);
            assert_eq!(serde_json::to_value(event).unwrap(), payload);
            assert_round_trip(event);

            let updated_at = aggregate.state.updated_at;
            update(&mut aggregate, expected);
            assert_price_confirmation(&aggregate);
            assert_eq!(aggregate.sequence, 4);
            assert_eq!(aggregate.uncommitted_events.len(), 4);
            assert!(aggregate.state.updated_at >= updated_at);
        }
    }
}

#[test]
fn mixed_changes_keep_field_order_and_preserve_other_state() {
    let original = input();
    let mut request = original.clone();
    request.identifiers.gtin = None;
    let mut aggregate = create(request);
    let created_at = aggregate.state.created_at;
    let mut identifiers = aggregate.state.identifiers.clone();
    identifiers.gtin = Some("new-gtin".into());
    identifiers.sku = Some("new-sku".into());
    identifiers.shopify_product_gid = None;
    identifiers.shopify_product_variant_gid = Some("new-variant".into());
    update(&mut aggregate, identifiers);
    assert_price_confirmation(&aggregate);
    assert_eq!(aggregate.sequence, 6);
    assert_eq!(aggregate.uncommitted_events.len(), 6);
    let expected_types = [
        "OfferGtinChanged",
        "OfferSkuChanged",
        "OfferShopifyProductGidChanged",
        "OfferShopifyProductVariantGidChanged",
        "OfferPriceConfirmed",
    ];
    for (index, (envelope, expected_type)) in aggregate.uncommitted_events[1..]
        .iter()
        .zip(expected_types)
        .enumerate()
    {
        assert_eq!(envelope.sequence(), index as u64 + 2);
        assert_eq!(envelope.event().event_type(), expected_type);
        assert_round_trip(envelope.event());
    }
    assert_eq!(aggregate.state.id, original.id);
    assert_eq!(aggregate.state.retailer_code(), &original.retailer_code);
    assert_eq!(aggregate.state.url(), original.url);
    assert_eq!(aggregate.state.created_at, created_at);
    assert_eq!(aggregate.state.target_market, original.target_market);
    assert_eq!(aggregate.state.language, original.language);
    assert_eq!(aggregate.state.content, original.content);
    assert_eq!(aggregate.state.price.last, original.price);
    assert_eq!(aggregate.state.price.history, [original.price]);
    assert_eq!(aggregate.state.availability, original.availability);
    assert_eq!(aggregate.state.item_condition, original.item_condition);
}

#[test]
fn exhausted_sequence_does_not_partially_apply_identifiers() {
    let mut aggregate = create(input());
    aggregate.sequence = u64::MAX - 1;
    let original_identifiers = aggregate.state.identifiers.clone();
    let updated_at = aggregate.state.updated_at;
    let mut identifiers = original_identifiers.clone();
    identifiers.gtin = None;
    identifiers.sku = None;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        update(&mut aggregate, identifiers);
    }));
    assert!(result.is_err());
    assert_eq!(aggregate.state.identifiers, original_identifiers);
    assert_eq!(aggregate.state.updated_at, updated_at);
    assert_eq!(aggregate.sequence, u64::MAX - 1);
    assert_eq!(aggregate.uncommitted_events.len(), 1);
}

#[test]
fn new_identifier_events_do_not_capture_legacy_identifier_payloads() {
    let payload = json!({
        "id": "offer-next",
        "gtin": "1234567890123",
        "sku": null,
        "external_identifiers": {
            "shopify_product_gid": null,
            "shopify_product_variation_gid": null,
            "shopify_handle": null
        }
    });
    assert!(serde_json::from_value::<RetailerOfferEvent>(payload).is_err());
}
