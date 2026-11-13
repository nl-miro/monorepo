use crate::graphql::raw::RawOfferDetails;
use serde_json::{Value, json};

/// The extracted contract keeps selected facts and intentionally omits raw metadata.
#[test]
fn extraction_preserves_selected_facts_and_round_trips() {
    for product in [include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    )] {
        let product: Value = serde_json::from_str(product).unwrap();
        for source in [
            json!({"schema_version":1,"product":product,"pages":[]}),
            json!({
                "schema_version":2,
                "acquired_at":"2026-11-12T00:00:00Z",
                "context":{"country":"CA","language":"FR","currency":"CAD"},
                "product":product,
                "pages":[],
            }),
        ] {
            let raw: RawOfferDetails = serde_json::from_value(source).unwrap();
            let expected = serde_json::to_value(&raw).unwrap();
            let extracted = super::prelude::ExtractedOfferDetails::try_from(raw).unwrap();
            let output = serde_json::to_value(&extracted).unwrap();
            assert_eq!(
                output["product"]["identifiers"]["id"],
                expected["product"]["id"]
            );
            assert_eq!(
                output["product"]["content"]["title"],
                expected["product"]["title"]
            );
            assert_eq!(output["context"], expected["context"]);
            assert_eq!(output["acquired_at"], expected["acquired_at"]);
            assert_eq!(
                output["product"]["variants"].as_array().unwrap().len(),
                expected["product"]["variants"]["edges"]
                    .as_array()
                    .unwrap()
                    .len()
            );
            for field in ["schema_version", "pages"] {
                assert!(output.get(field).is_none(), "{field}");
            }
            for field in [
                "__typename",
                "price_range",
                "compare_at_price_range",
                "variants_count",
            ] {
                assert!(output["product"].get(field).is_none(), "{field}");
            }
            assert!(output["collections"].is_array());
            assert!(output["images"].is_array());
            assert!(output["media"].is_array());
            let decoded: super::prelude::ExtractedOfferDetails =
                serde_json::from_value(output.clone()).unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), output);
        }
    }
}

/// Shopify date-times become UTC instants; offsets are normalised and junk is rejected.
#[test]
fn date_times_are_parsed_as_utc() {
    let product: Value = serde_json::from_str(include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    let mut source = json!({
        "schema_version":1,
        "acquired_at":"2026-11-12T09:10:11.123456789Z",
        "product":product,
        "pages":[],
    });
    source["product"]["updatedAt"] = json!("2026-11-12T10:00:00+02:00");
    let raw: RawOfferDetails = serde_json::from_value(source.clone()).unwrap();
    let extracted = super::prelude::ExtractedOfferDetails::try_from(raw).unwrap();
    assert_eq!(
        extracted.acquired_at.unwrap().to_rfc3339(),
        "2026-11-12T09:10:11.123456789+00:00"
    );
    assert_eq!(
        extracted.product.updated_at.to_rfc3339(),
        "2026-11-12T08:00:00+00:00"
    );
    for (field, value) in [("updatedAt", "yesterday"), ("createdAt", "2026-11-12")] {
        let mut source = source.clone();
        source["product"][field] = json!(value);
        let raw: RawOfferDetails = serde_json::from_value(source).unwrap();
        let error = super::prelude::ExtractedOfferDetails::try_from(raw).unwrap_err();
        assert!(error.contains("is not an RFC 3339 instant"), "{error}");
    }
}

#[test]
fn variant_count_is_validated_and_omitted() {
    let product: Value = serde_json::from_str(include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    let mut raw: RawOfferDetails = serde_json::from_value(json!({
        "schema_version": 1, "product": product, "pages": []
    }))
    .unwrap();
    let collected = i64::try_from(raw.product.variants.edges.len()).unwrap();
    raw.product.variants_count = Some(crate::graphql::raw::Count {
        type_name: "Count".to_string(),
        count: collected,
        precision: "EXACT".to_string(),
    });
    let extracted = super::prelude::ExtractedOfferDetails::try_from(raw.clone()).unwrap();
    let output = serde_json::to_value(extracted).unwrap();
    assert!(output["product"].get("variants_count").is_none());
    for expected in [collected - 1, collected + 1, -1] {
        raw.product.variants_count.as_mut().unwrap().count = expected;
        let error = super::prelude::ExtractedOfferDetails::try_from(raw.clone()).unwrap_err();
        assert!(error.contains("product.variants_count"), "{error}");
    }
}

#[test]
fn variant_count_lower_bound_and_missing_count() {
    let product: Value = serde_json::from_str(include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    let mut raw: RawOfferDetails = serde_json::from_value(json!({
        "schema_version": 1, "product": product, "pages": []
    }))
    .unwrap();
    let collected = i64::try_from(raw.product.variants.edges.len()).unwrap();
    raw.product.variants_count = None;
    super::prelude::ExtractedOfferDetails::try_from(raw.clone()).unwrap();
    for minimum in [0, collected, collected + 1] {
        raw.product.variants_count = Some(crate::graphql::raw::Count {
            type_name: "Count".to_string(),
            count: minimum,
            precision: "AT_LEAST".to_string(),
        });
        let result = super::prelude::ExtractedOfferDetails::try_from(raw.clone());
        if minimum <= collected {
            result.unwrap();
        } else {
            let error = result.unwrap_err();
            assert!(error.contains("AT_LEAST"), "{error}");
        }
    }
}

#[test]
fn default_options_are_filtered_without_losing_real_choices() {
    let raw: crate::graphql::raw::Product = serde_json::from_str(include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    for default_title in ["Default Title", "Default title"] {
        for variant_count in [1, 2] {
            let mut product = raw.clone();
            product.variants_count = None;
            product.variants.edges = vec![product.variants.edges[0].clone(); variant_count];
            product.options = serde_json::from_value(json!([
                {"__typename":"ProductOption", "id":"default", "name":"Title",
                 "optionValues":[{"__typename":"ProductOptionValue", "id":"default-value", "name":default_title,"swatch":null}]},
                {"__typename":"ProductOption", "id":"color", "name":"Color",
                 "optionValues":[{"__typename":"ProductOptionValue", "id":"color-value", "name":default_title,"swatch":null}]},
                {"__typename":"ProductOption", "id":"real-title", "name":"Title",
                 "optionValues":[{"__typename":"ProductOptionValue", "id":"real-value", "name":"Special Title","swatch":null}]},
                {"__typename":"ProductOption", "id":"multiple-values", "name":"Title",
                 "optionValues":[
                    {"__typename":"ProductOptionValue", "id":"default-value", "name":default_title,"swatch":null},
                    {"__typename":"ProductOptionValue", "id":"real-value", "name":"Special Title","swatch":null}
                 ]}
            ])).unwrap();
            for edge in &mut product.variants.edges {
                edge.node.selected_options = serde_json::from_value(json!([
                    {"__typename":"SelectedOption", "name":"Color", "value":"Blue"},
                    {"__typename":"SelectedOption", "name":"Title", "value":default_title},
                    {"__typename":"SelectedOption", "name":"Color", "value":default_title},
                    {"__typename":"SelectedOption", "name":"Title", "value":"Special Title"}
                ]))
                .unwrap();
            }
            let extracted = super::prelude::Product::try_from(product).unwrap();
            let ids: Vec<_> = extracted
                .options
                .iter()
                .map(|option| option.id.as_str())
                .collect();
            if variant_count == 1 {
                assert_eq!(ids, ["color", "real-title", "multiple-values"]);
            } else {
                assert_eq!(ids, ["default", "color", "real-title", "multiple-values"]);
            }
            for variant in extracted.variants {
                let options: Vec<_> = variant
                    .selected_options
                    .iter()
                    .map(|option| (option.name.as_str(), option.value.as_str()))
                    .collect();
                assert_eq!(
                    options,
                    [
                        ("Color", "Blue"),
                        ("Color", default_title),
                        ("Title", "Special Title")
                    ]
                );
            }
        }
    }
}

#[test]
fn default_only_options_become_empty_lists() {
    let mut product: crate::graphql::raw::Product = serde_json::from_str(include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    product.variants_count = None;
    product.variants.edges.truncate(1);
    product.options = serde_json::from_value(json!([
        {"__typename":"ProductOption", "id":"default", "name":"Title",
         "optionValues":[{"__typename":"ProductOptionValue", "id":"default-value", "name":"Default Title","swatch":null}]}
    ])).unwrap();
    product.variants.edges[0].node.selected_options = serde_json::from_value(json!([
        {"__typename":"SelectedOption", "name":"Title", "value":"Default Title"}
    ]))
    .unwrap();
    let output = serde_json::to_value(super::prelude::Product::try_from(product).unwrap()).unwrap();
    assert_eq!(output["options"], json!([]));
    assert_eq!(output["variants"][0]["selected_options"], json!([]));
}

#[test]
fn empty_unit_price_measurements_become_null_and_real_values_survive() {
    let product: crate::graphql::raw::Product = serde_json::from_str(include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    let variant = product.variants.edges[0].node.clone();
    let empty = json!({
        "__typename": "UnitPriceMeasurement", "measuredType": null,
        "quantityUnit": null, "quantityValue": 0, "referenceUnit": null,
        "referenceValue": 0
    });
    for measured_type in [Value::Null, json!("WEIGHT"), json!("VOLUME")] {
        for quantity in [json!(0), json!(0.0), json!(-0.0)] {
            let mut raw = variant.clone();
            let mut measurement = empty.clone();
            measurement["measuredType"] = measured_type.clone();
            measurement["quantityValue"] = quantity;
            raw.unit_price_measurement = Some(serde_json::from_value(measurement).unwrap());
            let output =
                serde_json::to_value(super::prelude::Variant::try_from(raw).unwrap()).unwrap();
            assert_eq!(output.get("unit_price_measurement"), Some(&Value::Null));
        }
    }
    let mut raw = variant.clone();
    raw.unit_price_measurement = None;
    assert!(
        super::prelude::Variant::try_from(raw)
            .unwrap()
            .unit_price_measurement
            .is_none()
    );
    for (field, value) in [
        ("quantityUnit", json!("ML")),
        ("quantityValue", json!(0.5)),
        ("referenceUnit", json!("L")),
        ("referenceValue", json!(1)),
    ] {
        let mut raw = variant.clone();
        let mut measurement = empty.clone();
        measurement[field] = value;
        let measurement: crate::graphql::raw::UnitPriceMeasurement =
            serde_json::from_value(measurement).unwrap();
        let expected = super::prelude::UnitPriceMeasurement::try_from(measurement.clone()).unwrap();
        raw.unit_price_measurement = Some(measurement);
        let extracted = super::prelude::Variant::try_from(raw).unwrap();
        assert_eq!(
            serde_json::to_value(extracted.unit_price_measurement).unwrap(),
            serde_json::to_value(expected).unwrap(),
            "{field}"
        );
    }
}

#[test]
fn extracted_money_uses_exact_currency_minor_units() {
    for (amount, currency, minor) in [
        ("79.99", "USD", 7999),
        ("1.2300", "EUR", 123),
        ("123.00", "JPY", 123),
        ("123.00", "KRW", 123),
        ("-0.01", "USD", -1),
        ("0", "USD", 0),
        ("92233720368547758.07", "USD", i64::MAX),
        ("-92233720368547758.08", "USD", i64::MIN),
    ] {
        let raw = crate::graphql::raw::Money {
            type_name: "MoneyV2".into(),
            amount: amount.into(),
            currency_code: currency.into(),
        };
        let extracted = money::Money::try_from(raw).unwrap();
        assert_eq!(
            extracted,
            money::Money::new(minor, currency.parse().unwrap())
        );
        let output = serde_json::to_value(&extracted).unwrap();
        assert_eq!(output, json!({"amount":minor,"currency":currency}));
        assert_eq!(
            serde_json::from_value::<money::Money>(output).unwrap(),
            extracted
        );
    }
    for (amount, currency) in [
        ("0.001", "USD"),
        ("1.1", "JPY"),
        ("NaN", "USD"),
        ("1.2.3", "USD"),
        ("", "USD"),
        ("1", "INVALID"),
        ("92233720368547758.08", "USD"),
        ("-92233720368547758.09", "USD"),
        ("999999999999999999999999999999999999999999", "USD"),
    ] {
        let raw = crate::graphql::raw::Money {
            type_name: "MoneyV2".into(),
            amount: amount.into(),
            currency_code: currency.into(),
        };
        assert!(money::Money::try_from(raw).is_err(), "{amount} {currency}");
    }
}

#[test]
fn variant_pricing_classifies_prices_and_serializes_the_enum() {
    let product: crate::graphql::raw::Product = serde_json::from_str(include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    let variant = product.variants.edges[0].node.clone();
    let raw_money = |amount: &str, currency: &str| crate::graphql::raw::Money {
        type_name: "MoneyV2".into(),
        amount: amount.into(),
        currency_code: currency.into(),
    };
    for (amount, comparison, discounted) in [
        ("10.00", None, false),
        ("10.00", Some(("15.00", "USD")), true),
        ("10.00", Some(("10.00", "USD")), false),
        ("10.00", Some(("5.00", "USD")), false),
        ("10.00", Some(("15.00", "EUR")), false),
        ("0", None, false),
        ("0", Some(("15.00", "USD")), false),
    ] {
        let mut raw = variant.clone();
        raw.price = raw_money(amount, "USD");
        raw.compare_at_price = comparison.map(|(amount, currency)| raw_money(amount, currency));
        let price = money::Money::try_from(raw.price.clone()).unwrap();
        let expected = if price.amount() == 0 {
            super::prelude::Pricing::Unknown
        } else if discounted {
            super::prelude::Pricing::Discounted(
                price,
                money::Money::try_from(raw.compare_at_price.clone().unwrap()).unwrap(),
            )
        } else {
            super::prelude::Pricing::Standard(price)
        };
        let extracted = super::prelude::Variant::try_from(raw).unwrap();
        assert_eq!(extracted.pricing, expected);
        let output = serde_json::to_value(&extracted).unwrap();
        assert!(output.get("price").is_none());
        assert!(output.get("compare_at_price").is_none());
        assert_eq!(output["pricing"], serde_json::to_value(&expected).unwrap());
        let decoded: super::prelude::Variant = serde_json::from_value(output).unwrap();
        assert_eq!(decoded.pricing, expected);
    }
    assert_eq!(
        serde_json::to_value(super::prelude::Pricing::Unknown).unwrap(),
        json!("Unknown")
    );
    let mut raw = variant;
    raw.price = raw_money("invalid", "USD");
    assert!(super::prelude::Variant::try_from(raw).is_err());
}

#[test]
fn collection_products_are_an_ordered_list_or_empty() {
    let base = json!({
        "__typename":"Collection", "id":"collection", "handle":"collection",
        "title":"Collection", "description":"", "descriptionHtml":"",
        "image":null, "onlineStoreUrl":null,
        "seo":{"__typename":"SEO", "title":null, "description":null},
        "updatedAt":"2026-11-12T00:00:00Z"
    });
    let nodes = json!([
        {"__typename":"Product", "id":"second", "handle":"second", "title":"Second"},
        {"__typename":"Product", "id":"first", "handle":"first", "title":"First"}
    ]);
    for products in [
        None,
        Some(Value::Null),
        Some(json!({
            "edges":[], "pageInfo":{"hasNextPage":false,"endCursor":null}
        })),
        Some(json!({
            "edges":[{"cursor":"a","node":nodes[0]}, {"cursor":"b","node":nodes[1]}],
            "pageInfo":{"hasNextPage":false,"endCursor":"b"}
        })),
    ] {
        let mut input = base.clone();
        if let Some(products) = products {
            input["products"] = products;
        }
        let expected = if input["products"]["edges"]
            .as_array()
            .is_some_and(|edges| !edges.is_empty())
        {
            json!([
                {"id":"second", "handle":"second", "title":"Second"},
                {"id":"first", "handle":"first", "title":"First"}
            ])
        } else {
            json!([])
        };
        let raw: crate::graphql::raw::Collection = serde_json::from_value(input).unwrap();
        let output =
            serde_json::to_value(super::prelude::Collection::try_from(raw).unwrap()).unwrap();
        assert_eq!(output["products"], expected);
    }
}

fn image_test_artifact() -> RawOfferDetails {
    let product: Value = serde_json::from_str(include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    serde_json::from_value(json!({"schema_version":1,"product":product,"pages":[]})).unwrap()
}

#[test]
fn root_images_are_unique_ordered_and_resolve_every_reference() {
    let mut raw = image_test_artifact();
    let first = raw.product.images.edges[0].clone();
    let mut second = first.clone();
    second.node.id = Some("second".into());
    let mut variant_image = first.node.clone();
    variant_image.id = Some("variant-only".into());
    let mut featured = first.node.clone();
    featured.id = Some("featured-only".into());
    raw.product.images.edges = vec![first.clone(), second, first.clone()];
    raw.product.variants.edges[0].node.image = Some(variant_image);
    raw.product.featured_image = Some(featured);
    let extracted = super::prelude::ExtractedOfferDetails::try_from(raw).unwrap();
    let ids: Vec<_> = extracted
        .images
        .iter()
        .map(|image| image.id.as_deref().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            first.node.id.as_deref().unwrap(),
            "second",
            "variant-only",
            "featured-only"
        ]
    );
    let references = extracted
        .product
        .content
        .images
        .iter()
        .chain(extracted.product.content.featured_image.iter())
        .chain(
            extracted
                .product
                .variants
                .iter()
                .filter_map(|variant| variant.image.as_ref()),
        );
    for reference in references {
        assert_eq!(ids.iter().filter(|id| **id == reference).count(), 1);
    }
}

#[test]
fn identical_images_across_gallery_variants_and_featured_are_shared() {
    let mut raw = image_test_artifact();
    let first = raw.product.images.edges[0].clone();
    raw.product.images.edges = vec![first.clone(), first.clone()];
    raw.product.featured_image = Some(first.node.clone());
    let variant = raw.product.variants.edges[0].clone();
    raw.product.variants.edges = vec![variant.clone(), variant];
    raw.product.variants_count = None;
    for edge in &mut raw.product.variants.edges {
        edge.node.image = Some(first.node.clone());
    }
    let extracted = super::prelude::ExtractedOfferDetails::try_from(raw).unwrap();
    assert_eq!(extracted.images.len(), 1);
    assert_eq!(extracted.images[0].id, first.node.id);
    for variant in extracted.product.variants {
        assert_eq!(variant.image, first.node.id);
    }
}

#[test]
fn conflicting_image_fields_fail_with_the_image_id() {
    for field in ["url", "alt_text", "width", "height", "thumbhash"] {
        for location in ["gallery", "variant", "featured"] {
            let mut raw = image_test_artifact();
            let first = raw.product.images.edges[0].clone();
            let mut conflicting = first.node.clone();
            match field {
                "url" => conflicting.url.push_str("?different"),
                "alt_text" => conflicting.alt_text = Some("different alt".into()),
                "width" => conflicting.width = Some(-1),
                "height" => conflicting.height = Some(-1),
                "thumbhash" => conflicting.thumbhash = Some("different hash".into()),
                _ => unreachable!(),
            }
            match location {
                "gallery" => {
                    let mut edge = first.clone();
                    edge.node = conflicting;
                    raw.product.images.edges.push(edge);
                }
                "variant" => raw.product.variants.edges[0].node.image = Some(conflicting),
                "featured" => raw.product.featured_image = Some(conflicting),
                _ => unreachable!(),
            }
            let error = super::prelude::ExtractedOfferDetails::try_from(raw).unwrap_err();
            assert!(
                error.contains("conflicting image records"),
                "{location} {field}: {error}"
            );
            assert!(error.contains(first.node.id.as_deref().unwrap()), "{error}");
        }
    }
}

#[test]
fn missing_image_ids_remain_errors() {
    for location in ["gallery", "variant", "featured"] {
        let mut raw = image_test_artifact();
        match location {
            "gallery" => raw.product.images.edges[0].node.id = None,
            "variant" => {
                raw.product.variants.edges[0]
                    .node
                    .image
                    .as_mut()
                    .unwrap()
                    .id = None
            }
            "featured" => raw.product.featured_image.as_mut().unwrap().id = None,
            _ => unreachable!(),
        }
        let error = super::prelude::ExtractedOfferDetails::try_from(raw).unwrap_err();
        assert!(error.contains("missing id"), "{location}: {error}");
    }
}
