//! Two requests and two responses per implementation, with no pagination or retries.
use retailer_sourcing::graphql;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

type Catalog = fn(&Path) -> Result<PathBuf, Box<dyn std::error::Error>>;
type Details = fn(&Path, &str) -> Result<PathBuf, Box<dyn std::error::Error>>;

#[test]
#[ignore = "runs six live Shopify requests and saves their responses"]
fn catalog_offers_and_offer_details_all_clients() {
    let output = std::env::var_os("GRAPHQL_OPERATIONS_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/artifacts/graphql-operations")
        });
    let clients: [(&str, Catalog, Details); 3] = [
        (
            "custom",
            graphql::catalog_offers_custom,
            graphql::offer_details_custom,
        ),
        (
            "graphql_client",
            graphql::catalog_offers_graphql_client,
            graphql::offer_details_graphql_client,
        ),
        (
            "cynic",
            graphql::catalog_offers_cynic,
            graphql::offer_details_cynic,
        ),
    ];
    let mut failures = vec![];
    for (implementation, catalog, details) in clients {
        for (operation, result) in [
            ("catalog_offers", catalog(&output)),
            (
                "offer_details",
                details(&output, "gid://shopify/Product/15395172516091"),
            ),
        ] {
            match result {
                Ok(folder) => {
                    println!("{implementation}/{operation}: {}", folder.display());
                    let request: Value =
                        serde_json::from_slice(&fs::read(folder.join("request.json")).unwrap())
                            .unwrap();
                    let response: Value =
                        serde_json::from_slice(&fs::read(folder.join("response.json")).unwrap())
                            .unwrap();
                    assert!(request["query"].is_string());
                    let result_present = match operation {
                        "catalog_offers" => {
                            response["data"]["products"]["nodes"].is_array()
                                && response["extensions"]["cost"]["requestedQueryCost"]
                                    .as_u64()
                                    .is_some_and(|cost| cost <= 1000)
                        }
                        "offer_details" => {
                            response["data"]["product"]["id"] == request["variables"]["id"]
                                && response["data"]["product"].is_object()
                        }
                        _ => unreachable!(),
                    };
                    if response["errors"]
                        .as_array()
                        .is_some_and(|errors| !errors.is_empty())
                        || !result_present
                    {
                        failures.push(format!(
                            "{implementation}/{operation}: valid result: {result_present}; errors: {}",
                            response["errors"]
                        ));
                    }
                    assert_eq!(fs::read_dir(folder).unwrap().count(), 2);
                }
                Err(error) => failures.push(format!("{implementation}/{operation}: {error}")),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
