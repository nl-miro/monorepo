use crate::graphql::raw::RawOfferDetails;
use serde_json::{Value, json};

/// Extraction reshapes raw data while retaining modeled product facts.
#[test]
fn extraction_preserves_product_facts_and_round_trips() {
    for product in [
        include_str!("../ankercom/offer_details/v2/fixtures/product.json"),
        include_str!("../ankercom/offer_details/v2/fixtures/expanded-product.json"),
    ] {
        let product: Value = serde_json::from_str(product).unwrap();
        for source in [
            json!({"schema_version":2,"context":{"country":"CA","language":"FR","currency":"CAD"},"acquired_at":"2026-11-12T00:00:00Z","product":product,"pages":[]}),
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
            assert!(output["product"].get("seo").is_none());
            assert_eq!(
                output["product"]["content"]["seo_title"],
                expected["product"]["seo"]["title"]
            );
            assert_eq!(
                output["product"]["content"]["seo_description"],
                expected["product"]["seo"]["description"]
            );

            assert_eq!(
                output["product"]["identifiers"]["id"],
                expected["product"]["id"]
            );
            assert_eq!(
                output["product"]["content"]["title"],
                expected["product"]["title"]
            );
            assert_eq!(
                output["product"]["total_inventory"],
                expected["product"]["totalInventory"]
            );
            assert_eq!(output["context"], expected["context"]);
            assert_eq!(
                output["product"]["variants"].as_array().unwrap().len(),
                expected["product"]["variants"]["edges"]
                    .as_array()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                output["collections"].as_array().unwrap().len(),
                expected["product"]["collections"]["edges"]
                    .as_array()
                    .unwrap()
                    .len()
            );
            for excluded in ["schema_version", "pages"] {
                assert!(output.get(excluded).is_none());
            }
            for excluded in ["__typename", "price_range", "compare_at_price_range"] {
                assert!(output["product"].get(excluded).is_none());
            }
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
        "../ankercom/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    let mut source = json!({
        "schema_version":2,"context":{"country":"CA","language":"FR","currency":"CAD"},
        "acquired_at":"2026-11-12T09:10:11.123456789Z",
        "product":product,
        "pages":[],
    });
    source["product"]["updatedAt"] = json!("2026-11-12T10:00:00+02:00");
    let raw: RawOfferDetails = serde_json::from_value(source.clone()).unwrap();
    let extracted = super::prelude::ExtractedOfferDetails::try_from(raw).unwrap();
    assert_eq!(
        extracted.acquired_at.to_rfc3339(),
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
        "../ankercom/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    let mut raw: RawOfferDetails = serde_json::from_value(json!({
        "schema_version":2,"context":{"country":"CA","language":"FR","currency":"CAD"},"acquired_at":"2026-11-12T00:00:00Z", "product": product, "pages": []
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
        "../ankercom/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    let mut raw: RawOfferDetails = serde_json::from_value(json!({
        "schema_version":2,"context":{"country":"CA","language":"FR","currency":"CAD"},"acquired_at":"2026-11-12T00:00:00Z", "product": product, "pages": []
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
        "../ankercom/offer_details/v2/fixtures/product.json"
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
        "../ankercom/offer_details/v2/fixtures/product.json"
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
        "../ankercom/offer_details/v2/fixtures/product.json"
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
        "../ankercom/offer_details/v2/fixtures/product.json"
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
    for amount in ["10.00", "0"] {
        for comparison in ["15.00", "10.00", "5.00"] {
            let mut raw = variant.clone();
            raw.price = raw_money(amount, "USD");
            raw.compare_at_price = Some(raw_money(comparison, "EUR"));
            let id = raw.id.clone();
            let error = super::prelude::Variant::try_from(raw).unwrap_err();
            assert!(error.contains(&id), "{error}");
            assert!(
                error.contains("compare_at_price")
                    && error.contains("EUR")
                    && error.contains("USD"),
                "{error}"
            );
        }
    }
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

#[test]
fn root_images_are_unique_ordered_and_references_resolve() {
    let product: Value = serde_json::from_str(include_str!(
        "../ankercom/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    let mut raw: RawOfferDetails = serde_json::from_value(json!({
        "schema_version":2,"context":{"country":"CA","language":"FR","currency":"CAD"},"acquired_at":"2026-11-12T00:00:00Z", "product": product, "pages": []
    }))
    .unwrap();
    let original = raw.product.images.edges[0].clone();
    raw.product.images.edges = vec![original.clone(), original.clone()];
    let mut variant_image = original.node.clone();
    variant_image.id = Some("variant-only".into());
    raw.product.variants.edges[0].node.image = Some(variant_image.clone());
    let mut featured_image = original.node.clone();
    featured_image.id = Some("featured-only".into());
    raw.product.featured_image = Some(featured_image);
    let extracted = super::prelude::ExtractedOfferDetails::try_from(raw.clone()).unwrap();
    let ids: Vec<_> = extracted
        .images
        .iter()
        .map(|image| image.id.as_deref().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            original.node.id.as_deref().unwrap(),
            "variant-only",
            "featured-only"
        ]
    );
    for id in extracted
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
        )
    {
        assert_eq!(
            extracted
                .images
                .iter()
                .filter(|image| image.id.as_ref() == Some(id))
                .count(),
            1
        );
    }
    // The same image in product, variant and featured positions is one root record.
    raw.product.variants.edges[0].node.image = Some(original.node.clone());
    raw.product.featured_image = Some(original.node.clone());
    let extracted = super::prelude::ExtractedOfferDetails::try_from(raw.clone()).unwrap();
    assert_eq!(extracted.images.len(), 1);
    assert_eq!(
        extracted.acquired_at.to_rfc3339(),
        "2026-11-12T00:00:00+00:00"
    );
    for location in ["root", "variant", "featured"] {
        let mut changed = raw.clone();
        let image = match location {
            "root" => &mut changed.product.images.edges[1].node,
            "variant" => changed.product.variants.edges[0]
                .node
                .image
                .as_mut()
                .unwrap(),
            _ => changed.product.featured_image.as_mut().unwrap(),
        };
        image.alt_text = Some("conflicting observation".into());
        let error = super::prelude::ExtractedOfferDetails::try_from(changed).unwrap_err();
        assert!(
            error.contains(original.node.id.as_deref().unwrap()),
            "{error}"
        );
        assert!(error.contains("conflicting image"), "{error}");
    }
    raw.product.images.edges[0].node.id = None;
    assert!(
        super::prelude::ExtractedOfferDetails::try_from(raw)
            .unwrap_err()
            .contains("missing id")
    );
}

#[test]
fn missing_context_is_recovered_only_from_consistent_source_evidence() {
    let product: Value = serde_json::from_str(include_str!(
        "../ankercom/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    let source = json!({"schema_version":1,"acquired_at":"2026-11-12T00:00:00Z",
    "product":product,"pages":[{
        "request":{"query":"query Test { product { id } }","operationName":"Test","variables":{}},
        "response":{"extensions":{"context":{"country":"US","language":"EN"}}}
    }]});
    let raw: RawOfferDetails = serde_json::from_value(source).unwrap();
    let expected_currency = raw
        .product
        .price_range
        .min_variant_price
        .currency_code
        .clone();
    let extracted = super::prelude::ExtractedOfferDetails::try_from(raw.clone()).unwrap();
    assert_eq!(extracted.context.country, "US");
    assert_eq!(extracted.context.language, "EN");
    assert_eq!(extracted.context.currency, expected_currency);
    let output = serde_json::to_value(extracted).unwrap();
    let _: super::prelude::ExtractedOfferDetails = serde_json::from_value(output).unwrap();

    // Empty variant lists still carry observed currency in product price ranges.
    let mut empty = raw.clone();
    empty.product.variants.edges.clear();
    empty.product.variants_count = None;
    assert_eq!(
        super::prelude::ExtractedOfferDetails::try_from(empty)
            .unwrap()
            .context
            .currency,
        expected_currency
    );
    for field in ["country", "language"] {
        for value in [Value::Null, json!(""), json!(123)] {
            let mut changed = raw.clone();
            changed.pages[0].response["extensions"]["context"][field] = value;
            assert!(
                super::prelude::ExtractedOfferDetails::try_from(changed)
                    .unwrap_err()
                    .contains(field)
            );
        }
        let mut missing = raw.clone();
        missing.pages[0].response["extensions"]["context"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            super::prelude::ExtractedOfferDetails::try_from(missing)
                .unwrap_err()
                .contains(field)
        );
        let mut conflict = raw.clone();
        conflict.pages.push(conflict.pages[0].clone());
        conflict.pages[1].response["extensions"]["context"][field] = json!("OTHER");
        assert!(
            super::prelude::ExtractedOfferDetails::try_from(conflict)
                .unwrap_err()
                .contains(field)
        );
    }
    let mut conflict = raw.clone();
    conflict.product.price_range.max_variant_price.currency_code = "OTHER".into();
    assert!(
        super::prelude::ExtractedOfferDetails::try_from(conflict)
            .unwrap_err()
            .contains("currency")
    );
    let mut missing = raw.clone();
    missing.pages.clear();
    assert!(
        super::prelude::ExtractedOfferDetails::try_from(missing)
            .unwrap_err()
            .contains("context")
    );
    let mut missing_time = raw.clone();
    missing_time.acquired_at = None;
    assert!(
        super::prelude::ExtractedOfferDetails::try_from(missing_time)
            .unwrap_err()
            .contains("acquired_at")
    );
    // An explicit context remains authoritative; fallback evidence is not consulted.
    let mut explicit = raw;
    explicit.context = Some(retailer_sauce::shopify::markets::MarketContext {
        country: "CA".into(),
        language: "FR".into(),
        currency: "CAD".into(),
    });
    explicit.pages[0].response["extensions"]["context"]["country"] = json!(123);
    assert_eq!(
        super::prelude::ExtractedOfferDetails::try_from(explicit)
            .unwrap()
            .context
            .country,
        "CA"
    );
}

#[test]
fn checkout_money_uses_validated_minor_units() {
    use super::prelude::CheckoutChargeValue;
    for (amount, currency, minor) in [
        ("79.99", "USD", 7999),
        ("123", "JPY", 123),
        ("1.2300", "EUR", 123),
        ("12.34", "SAR", 1234),
    ] {
        let raw = crate::graphql::raw::CheckoutChargeValue::MoneyV2 {
            amount: amount.into(),
            currency_code: currency.into(),
        };
        let extracted = CheckoutChargeValue::try_from(raw).unwrap();
        assert!(matches!(&extracted, CheckoutChargeValue::Money(value)
            if value.amount() == minor && value.currency().as_str() == currency));
        let output = serde_json::to_value(extracted).unwrap();
        assert_eq!(
            output,
            json!({"__typename":"Money","amount":minor,"currency":currency})
        );
        let decoded: CheckoutChargeValue = serde_json::from_value(output.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), output);
    }
    for (amount, currency) in [
        ("junk", "USD"),
        ("1.001", "USD"),
        ("1", "ZZZ"),
        ("9223372036854775808", "JPY"),
    ] {
        let raw = crate::graphql::raw::CheckoutChargeValue::MoneyV2 {
            amount: amount.into(),
            currency_code: currency.into(),
        };
        assert!(
            CheckoutChargeValue::try_from(raw).is_err(),
            "{amount} {currency}"
        );
    }
    let raw = crate::graphql::raw::CheckoutChargeValue::SellingPlanCheckoutChargePercentageValue {
        percentage: serde_json::from_value(json!(12.5)).unwrap(),
    };
    let output = serde_json::to_value(CheckoutChargeValue::try_from(raw).unwrap()).unwrap();
    assert_eq!(
        output,
        json!({"__typename":"SellingPlanCheckoutChargePercentageValue","percentage":12.5})
    );
}
