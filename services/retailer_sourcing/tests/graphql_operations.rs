//! Two requests and two responses per implementation and storefront, without retries.
use retailer_sourcing::graphql::{self, Storefront};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

type Catalog = fn(Storefront, &Path) -> Result<PathBuf, Box<dyn std::error::Error>>;
type Details = fn(Storefront, &Path, &str) -> Result<PathBuf, Box<dyn std::error::Error>>;

#[test]
fn saved_au_pairs_pass_response_validation() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/artifacts/graphql-operations/ankerau");
    for implementation in ["custom", "graphql_client", "cynic"] {
        for operation in ["catalog_offers", "offer_details"] {
            validate(
                &root.join(implementation).join(operation),
                operation,
                Storefront::AnkerAu,
                "gid://shopify/Product/15395172516091",
            )
            .unwrap();
        }
    }
}

#[test]
#[ignore = "runs six live Shopify requests and saves their responses"]
fn ankerau_catalog_offers_and_offer_details_all_clients() {
    run(Storefront::AnkerAu, "gid://shopify/Product/15395172516091");
}

#[test]
#[ignore = "runs six live Shopify requests and saves their responses"]
fn ankerca_catalog_offers_and_offer_details_all_clients() {
    // Existing Canada fixture; confirm against the live catalog before accepting new artifacts.
    run(Storefront::AnkerCa, "gid://shopify/Product/8302268514493");
}

fn price_valid(price: &Value, currency: &str) -> bool {
    price["amount"].as_str().is_some_and(|amount| {
        !amount.is_empty()
            && amount
                .parse::<f64>()
                .is_ok_and(|n| n.is_finite() && n >= 0.0)
    }) && price["currencyCode"] == currency
}

fn validate(
    folder: &Path,
    operation: &str,
    storefront: Storefront,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut documents = Vec::new();
    for file in ["request.json", "response.json"] {
        let bytes = fs::read(folder.join(file))?;
        let value: Value = serde_json::from_slice(&bytes)?;
        if bytes != serde_json::to_vec_pretty(&value)? {
            return Err(format!("{file} is not pretty JSON").into());
        }
        documents.push(value);
    }
    let request = &documents[0];
    let response = &documents[1];
    if response["errors"]
        .as_array()
        .is_some_and(|errors| !errors.is_empty())
    {
        return Err(format!("GraphQL errors: {}", response["errors"]).into());
    }
    if !request["query"].is_string()
        || response["extensions"]["context"]["country"] != storefront.country()
        || response["extensions"]["context"]["language"] != "EN"
    {
        return Err("missing query or incorrect storefront context".into());
    }
    if operation == "catalog_offers" {
        let nodes = response["data"]["products"]["nodes"]
            .as_array()
            .ok_or("missing products")?;
        if nodes.is_empty()
            || nodes.len() > 250
            || !response["extensions"]["cost"]["requestedQueryCost"]
                .as_u64()
                .is_some_and(|cost| cost <= 1000)
        {
            return Err("invalid catalog size or tokenless cost".into());
        }
        for product in nodes {
            if !product["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("gid://shopify/Product/"))
                || !product["availableForSale"].is_boolean()
                || !price_valid(
                    &product["priceRange"]["minVariantPrice"],
                    storefront.currency(),
                )
                || !price_valid(
                    &product["priceRange"]["maxVariantPrice"],
                    storefront.currency(),
                )
            {
                return Err("invalid product identity, availability or prices".into());
            }
            let variants = product["variants"]["edges"]
                .as_array()
                .ok_or("missing variants")?;
            if variants.len() > 20 {
                return Err("too many variants".into());
            }
            for edge in variants {
                let variant = &edge["node"];
                if !variant["id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("gid://shopify/ProductVariant/"))
                    || !variant["availableForSale"].is_boolean()
                    || !price_valid(&variant["price"], storefront.currency())
                {
                    return Err("invalid variant identity, availability or price".into());
                }
            }
        }
    } else {
        let product = &response["data"]["product"];
        if product["id"] != id
            || !product["availableForSale"].is_boolean()
            || !price_valid(
                &product["priceRange"]["minVariantPrice"],
                storefront.currency(),
            )
        {
            return Err("missing requested product or invalid product data".into());
        }
    }
    if fs::read_dir(folder)?.count() != 2 {
        return Err("expected exactly one request and response".into());
    }
    Ok(())
}

fn run(storefront: Storefront, product_id: &str) {
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
    let mut failures = Vec::new();
    for (implementation, catalog, details) in clients {
        for (operation, result) in [
            ("catalog_offers", catalog(storefront, &output)),
            ("offer_details", details(storefront, &output, product_id)),
        ] {
            let result = result.and_then(|folder| {
                let expected = output
                    .join(storefront.slug())
                    .join(implementation)
                    .join(operation);
                if folder != expected {
                    return Err("incorrect artifact folder".into());
                }
                validate(&folder, operation, storefront, product_id)?;
                println!(
                    "{}/{implementation}/{operation}: {}",
                    storefront.slug(),
                    folder.display()
                );
                Ok(())
            });
            if let Err(error) = result {
                failures.push(format!(
                    "{}/{implementation}/{operation}: {error}",
                    storefront.slug()
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
