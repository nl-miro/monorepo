use super::*;
use serde_json::json;
use std::cell::RefCell;

struct Reply {
    body: Value,
    calls: RefCell<Vec<GraphqlRequest>>,
}
impl Http for Reply {
    fn post(
        &self,
        _: &str,
        request: &GraphqlRequest,
        _: Option<&StorefrontToken>,
    ) -> Result<Value, String> {
        self.calls.borrow_mut().push(request.clone());
        Ok(self.body.clone())
    }
}
fn engine() -> Engine {
    Engine {
        code: "test",
        config: StorefrontConfig {
            acquire_token: false,
            token_required: false,
            storefront_url: "https://example.com",
            graphql_url: "https://example.com/graphql",
            token_market: None,
            token_environment: None,
            country: "AD",
            language: "EN",
            currency: "EUR",
        },
    }
}
#[test]
fn omitted_catalog_country_accepts_shop_default_but_checks_language() {
    let mut reply = Reply {
        body: json!({"data":{"products":{"edges":[],"pageInfo":{"hasNextPage":false,"endCursor":null}}},"extensions":{"context":{"country":"US","language":"EN"}}}),
        calls: RefCell::new(vec![]),
    };
    engine().catalog(&reply, None, None, None).unwrap();
    assert!(reply.calls.borrow()[0].variables.get("country").is_none());
    reply.body["extensions"]["context"]["language"] = json!("FR");
    assert!(engine().catalog(&reply, None, None, None).is_err());
}
#[test]
fn omitted_detail_country_uses_observed_currency_not_configured_default() {
    let mut product: Value = serde_json::from_str(include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    for field in ["collections", "images", "media", "variants"] {
        product[field] = json!({"edges":[],"pageInfo":{"hasNextPage":false,"endCursor":null}});
    }
    fn currency(value: &mut Value) {
        match value {
            Value::Object(map) => {
                if map.contains_key("currencyCode") {
                    map.insert("currencyCode".into(), json!("USD"));
                }
                for value in map.values_mut() {
                    currency(value);
                }
            }
            Value::Array(values) => values.iter_mut().for_each(currency),
            _ => {}
        }
    }
    currency(&mut product);
    let id = product["id"].as_str().unwrap().to_owned();
    let mut reply = Reply {
        body: json!({"data":{"product":product},"extensions":{"context":{"country":"US","language":"EN"}}}),
        calls: RefCell::new(vec![]),
    };
    let artifact: Value =
        serde_json::from_str(&engine().details(&reply, None, None, &id).unwrap()).unwrap();
    assert_eq!(artifact["pages"][0]["response"], reply.body);
    assert!(
        reply
            .calls
            .borrow()
            .iter()
            .all(|request| request.variables.get("country").is_none())
    );
    // Omitting country must not permit mixed-currency product prices.
    reply.body["data"]["product"]["priceRange"]["minVariantPrice"]["currencyCode"] = json!("EUR");
    assert!(engine().details(&reply, None, None, &id).is_err());
    reply.body["data"]["product"]["priceRange"]["minVariantPrice"]["currencyCode"] = json!("USD");
    // Explicit context continues to require its own country and currency.
    let context = MarketContext {
        country: "AD".into(),
        language: "EN".into(),
        currency: "EUR".into(),
    };
    assert!(engine().details(&reply, None, Some(&context), &id).is_err());
    reply.body["extensions"]["context"]["country"] = json!("AD");
    assert!(engine().details(&reply, None, Some(&context), &id).is_err());
}

#[test]
fn inventory_language_country_and_currency_are_preserved_through_pagination() {
    use std::collections::VecDeque;
    struct Pages {
        pages: RefCell<VecDeque<Value>>,
        calls: RefCell<Vec<GraphqlRequest>>,
    }
    impl Http for Pages {
        fn post(
            &self,
            _: &str,
            request: &GraphqlRequest,
            _: Option<&StorefrontToken>,
        ) -> Result<Value, String> {
            self.calls.borrow_mut().push(request.clone());
            Ok(self
                .pages
                .borrow_mut()
                .pop_front()
                .expect("unexpected request"))
        }
    }
    let context = RequestedContext {
        country: None,
        language: "DE".into(),
        currency: None,
        observed_country: None,
    };
    let response = |country: &str, more: bool| json!({"data":{"products":{"edges":[],"pageInfo":{"hasNextPage":more,"endCursor":"next"}}},"extensions":{"context":{"country":country,"language":"DE"}}});
    for next_country in ["US", "CA"] {
        let http = Pages {
            pages: RefCell::new([response("US", true), response(next_country, false)].into()),
            calls: RefCell::new(vec![]),
        };
        let result = engine().catalog_for_locale(&http, None, Some(&context), None);
        assert_eq!(result.is_ok(), next_country == "US");
        if let Ok(result) = result {
            assert_eq!(
                result.context.unwrap().observed_country.as_deref(),
                Some("US")
            );
        }
        assert_eq!(http.calls.borrow().len(), 2);
        for call in http.calls.borrow().iter() {
            assert!(call.variables.get("country").is_none());
            assert_eq!(call.variables["language"], "DE");
        }
    }
    let mut p: Value = serde_json::from_str(include_str!(
        "../minisforumau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    for field in ["collections", "images", "media", "variants"] {
        p[field] = json!({"edges":[],"pageInfo":{"hasNextPage":false,"endCursor":null}});
    }
    p["images"]["pageInfo"] = json!({"hasNextPage":true,"endCursor":"image-next"});
    let id = p["id"].as_str().unwrap().to_owned();
    let first =
        json!({"data":{"product":p},"extensions":{"context":{"country":"US","language":"DE"}}});
    for next_country in ["US", "CA"] {
        let next = json!({"data":{"product":{"id":id,"images":{"edges":[],"pageInfo":{"hasNextPage":false,"endCursor":null}}}},"extensions":{"context":{"country":next_country,"language":"DE"}}});
        let http = Pages {
            pages: RefCell::new([first.clone(), next].into()),
            calls: RefCell::new(vec![]),
        };
        let result = engine().details_for_locale(&http, None, Some(&context), &id);
        assert_eq!(result.is_ok(), next_country == "US");
        if let Ok(result) = result {
            let artifact: Value = serde_json::from_str(&result.body).unwrap();
            let observed = result.context.unwrap();
            assert_eq!(observed.country, "US");
            assert_eq!(observed.language, "DE");
            assert_eq!(artifact["context"], serde_json::to_value(observed).unwrap());
            assert_eq!(artifact["schema_version"], 2);
        }
        for call in http.calls.borrow().iter() {
            assert!(call.variables.get("country").is_none());
            assert_eq!(call.variables["language"], "DE");
        }
    }
    let mut wrong_language = Reply {
        body: first.clone(),
        calls: RefCell::new(vec![]),
    };
    wrong_language.body["extensions"]["context"]["language"] = json!("EN");
    assert!(
        engine()
            .details_for_locale(&wrong_language, None, Some(&context), &id)
            .is_err()
    );
    let mut wrong_country = context.clone();
    wrong_country.country = Some("DE".into());
    assert!(
        engine()
            .details_for_locale(
                &Reply {
                    body: first.clone(),
                    calls: RefCell::new(vec![])
                },
                None,
                Some(&wrong_country),
                &id
            )
            .is_err()
    );
    let mut mixed = first;
    mixed["data"]["product"]["images"]["pageInfo"]["hasNextPage"] = json!(false);
    mixed["data"]["product"]["priceRange"]["minVariantPrice"]["currencyCode"] = json!("JPY");
    assert!(
        engine()
            .details_for_locale(
                &Reply {
                    body: mixed,
                    calls: RefCell::new(vec![])
                },
                None,
                Some(&context),
                &id
            )
            .is_err()
    );
}

#[test]
fn upstream_errors_survive_partial_data_decoding_and_redact_access_tokens() {
    let token = StorefrontToken::new("synthetic-error-token".into()).unwrap();
    let reply = Reply {
        body: json!({"data":{"product":{}},"errors":[{
            "message":"Denied synthetic-error-token", "path":["product","sellingPlanGroups"],
            "extensions":{"code":"ACCESS_DENIED","requiredAccess":"unauthenticated_read_selling_plans"}
        }]}),
        calls: RefCell::new(vec![]),
    };
    let error = engine()
        .details(&reply, Some(&token), None, "gid://shopify/Product/1")
        .unwrap_err();
    for part in [
        "OfferDetails",
        "ACCESS_DENIED",
        "sellingPlanGroups",
        "unauthenticated_read_selling_plans",
        "[redacted]",
    ] {
        assert!(error.contains(part), "{error}");
    }
    assert!(!error.contains(token.as_str()));
    let catalog = engine()
        .catalog(&reply, Some(&token), None, None)
        .err()
        .unwrap();
    assert!(catalog.contains("CatalogIdentifiers"));
    let missing = Reply {
        body: json!({"data":null}),
        calls: RefCell::new(vec![]),
    };
    let error = engine()
        .details(&missing, None, None, "gid://shopify/Product/1")
        .unwrap_err();
    assert!(error.contains("missing or null data without GraphQL errors"));
}

#[test]
fn legacy_market_discovery_does_not_fetch() {
    struct NoHttp;
    impl Http for NoHttp {}
    let error = engine().discover_markets(&NoHttp, None).unwrap_err();
    assert!(error.contains("use Retailer.supported_locales"));
}
