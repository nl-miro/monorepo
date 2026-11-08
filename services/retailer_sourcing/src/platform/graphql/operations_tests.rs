//! Compare the actual request builders with the human-readable Bruno requests.
use super::{Result, Storefront, custom, cynic, graphql_client};
use serde_json::{Value, json};

type Catalog = fn(Storefront) -> Result<Value>;
type Details = fn(Storefront, &str) -> Result<Value>;
type Http = fn(Storefront, &Value, Option<&str>) -> Result<reqwest::blocking::RequestBuilder>;

fn normalized_query(query: &str) -> Value {
    fn normalize(value: &mut Value) {
        match value {
            Value::Object(fields) => {
                fields.remove("pos");
                fields.values_mut().for_each(normalize);
                // Selection/argument order is irrelevant, but literal list order is not.
                for key in ["items", "arguments", "variable_definitions"] {
                    if let Some(Value::Array(items)) = fields.get_mut(key) {
                        items.sort_by_key(Value::to_string);
                        if key == "items" {
                            items.dedup();
                        }
                    }
                }
            }
            Value::Array(items) => items.iter_mut().for_each(normalize),
            _ => (),
        }
    }
    let document = async_graphql::parser::parse_query(query).unwrap();
    let mut value = serde_json::to_value(document).unwrap();
    normalize(&mut value);
    value
}

fn block<'a>(bruno: &'a str, name: &str) -> &'a str {
    bruno
        .split_once(&format!("{name} {{\n"))
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0
}

#[test]
fn all_requests_match_bruno_for_both_storefronts() {
    let clients: [(&str, Catalog, Details, Http); 3] = [
        (
            "custom",
            custom::catalog_request,
            custom::details_request,
            custom::http_request,
        ),
        (
            "graphql_client",
            graphql_client::catalog_request,
            graphql_client::details_request,
            graphql_client::http_request,
        ),
        (
            "cynic",
            cynic::catalog_request,
            cynic::details_request,
            cynic::http_request,
        ),
    ];
    for (storefront, catalog_bruno, details_bruno) in [
        (
            Storefront::AnkerAu,
            include_str!("../../../../../bruno/ankerau-catalog/catalog-offers.bru"),
            include_str!("../../../../../bruno/ankerau-catalog/offer-details.bru"),
        ),
        (
            Storefront::AnkerCa,
            include_str!("../../../../../bruno/ankerca-catalog/catalog-offers.bru"),
            include_str!("../../../../../bruno/ankerca-catalog/offer-details.bru"),
        ),
    ] {
        let variables: Value =
            serde_json::from_str(block(details_bruno, "body:graphql:vars")).unwrap();
        let id = variables["id"].as_str().unwrap();
        for (name, catalog, details, http) in clients {
            for (bruno, request, operation, token) in [
                (
                    catalog_bruno,
                    catalog(storefront).unwrap(),
                    "CatalogOffers",
                    None,
                ),
                (
                    details_bruno,
                    details(storefront, id).unwrap(),
                    "OfferDetails",
                    Some(storefront.token()),
                ),
            ] {
                assert_eq!(
                    normalized_query(request["query"].as_str().unwrap()),
                    normalized_query(block(bruno, "body:graphql")),
                    "{storefront:?}/{name}/{operation}"
                );
                assert_eq!(request["operationName"], operation);
                let variables: Value =
                    serde_json::from_str(block(bruno, "body:graphql:vars")).unwrap();
                assert_eq!(request["variables"], variables);
                assert!(block(bruno, "meta").contains("type: graphql"));
                let built = http(storefront, &request, token).unwrap().build().unwrap();
                assert_eq!(built.method(), reqwest::Method::POST);
                let url = block(bruno, "post")
                    .lines()
                    .find_map(|line| line.trim().strip_prefix("url: "))
                    .unwrap();
                assert_eq!(built.url().as_str(), url);
                assert_eq!(built.headers()["Content-Type"], "application/json");
                let header = block(bruno, "headers").lines().find_map(|line| {
                    line.trim()
                        .strip_prefix("X-Shopify-Storefront-Access-Token: ")
                });
                assert_eq!(header, token);
                assert_eq!(
                    built
                        .headers()
                        .get("X-Shopify-Storefront-Access-Token")
                        .map(|v| v.to_str().unwrap()),
                    header
                );
                assert_eq!(
                    serde_json::from_slice::<Value>(built.body().unwrap().as_bytes().unwrap())
                        .unwrap(),
                    request
                );
            }
        }
    }
}

#[test]
fn canada_matches_au_selections_and_changes_only_country() {
    for (au, ca) in [
        (
            include_str!("ankerau/catalog_offers.graphql"),
            include_str!("ankerca/catalog_offers.graphql"),
        ),
        (
            include_str!("ankerau/offer_details.graphql"),
            include_str!("ankerca/offer_details.graphql"),
        ),
    ] {
        assert_eq!(
            normalized_query(&au.replace("country: AU", "country: CA")),
            normalized_query(ca)
        );
    }
    assert_ne!(
        Storefront::AnkerAu.endpoint(),
        Storefront::AnkerCa.endpoint()
    );
    assert_ne!(Storefront::AnkerAu.token(), Storefront::AnkerCa.token());
    assert_eq!(Storefront::AnkerCa.country(), "CA");
    assert_eq!(Storefront::AnkerCa.currency(), "CAD");
    assert_eq!(
        custom::catalog_request(Storefront::AnkerCa).unwrap()["variables"],
        json!({"after":null})
    );
}

#[test]
fn query_comparison_preserves_values_and_fragment_conditions() {
    assert_eq!(
        normalized_query("{ x { a b } }"),
        normalized_query("{ x { b a } }")
    );
    assert_ne!(
        normalized_query("{ x(first:20) { a } }"),
        normalized_query("{ x(first:5) { a } }")
    );
    assert_ne!(
        normalized_query("{ x { ... on A { a } } }"),
        normalized_query("{ x { ... on B { a } } }")
    );
}
