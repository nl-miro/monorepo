//! Keep the fixed request documents aligned with their Bruno originals.
use serde_json::Value;

#[test]
fn offer_details_token_matches_bruno() {
    let bruno = include_str!("../../../../../bruno/ankerau-catalog/offer-details.bru");
    let token = bruno
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("X-Shopify-Storefront-Access-Token: ")
        })
        .unwrap();
    assert_eq!(super::STOREFRONT_TOKEN, token);
}

fn normalized_query(query: &str) -> Value {
    fn remove_positions(value: &mut Value) {
        match value {
            Value::Object(fields) => {
                fields.remove("pos");
                fields.values_mut().for_each(remove_positions);
            }
            Value::Array(items) => items.iter_mut().for_each(remove_positions),
            _ => (),
        }
    }
    let document = async_graphql::parser::parse_query(query).unwrap();
    let mut value = serde_json::to_value(document).unwrap();
    remove_positions(&mut value);
    value
}

#[test]
fn fixed_queries_match_bruno() {
    for (bruno, document) in [
        (
            include_str!("../../../../../bruno/ankerau-catalog/catalog-offers.bru"),
            include_str!("ankerau/catalog_offers.graphql"),
        ),
        (
            include_str!("../../../../../bruno/ankerau-catalog/offer-details.bru"),
            include_str!("ankerau/offer_details.graphql"),
        ),
    ] {
        let body = bruno
            .split("body:graphql {\n")
            .nth(1)
            .unwrap()
            .split_once("\n}")
            .unwrap()
            .0;
        assert_eq!(normalized_query(document), normalized_query(body));
    }
}
