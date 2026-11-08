//! Execute each review query once and save its request and unmodified response.
use super::{Result, save};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

fn execute(output: &Path, name: &str, request: Value) -> Result<PathBuf> {
    let folder = output.join("graphql_client").join(name);
    save(&folder.join("request.json"), &request)?;
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .build()?
        .post("https://oneanker-au.myshopify.com/api/2026-10/graphql.json")
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&request)?)
        .send()?;
    let status = response.status();
    std::fs::write(folder.join("response.json"), response.bytes()?)?;
    if !status.is_success() {
        return Err(format!("HTTP {status}; response saved in {}", folder.display()).into());
    }
    Ok(folder)
}
#[allow(clippy::upper_case_acronyms)]
type HTML = String;
#[allow(clippy::upper_case_acronyms)]
type URL = String;
type DateTime = String;
type Decimal = String;
type Color = String;

#[derive(::graphql_client::GraphQLQuery)]
#[graphql(
    schema_path = "src/platform/graphql/schema.graphql",
    query_path = "src/platform/graphql/catalog_offers.graphql",
    response_derives = "Debug"
)]
struct CatalogOffers;
#[derive(::graphql_client::GraphQLQuery)]
#[graphql(
    schema_path = "src/platform/graphql/schema.graphql",
    query_path = "src/platform/graphql/offer_details.graphql",
    response_derives = "Debug"
)]
struct OfferDetails;

pub(crate) fn catalog_offers(output: &Path) -> Result<PathBuf> {
    use ::graphql_client::GraphQLQuery;
    execute(
        output,
        "catalog_offers",
        serde_json::to_value(CatalogOffers::build_query(catalog_offers::Variables {
            after: None,
        }))?,
    )
}
pub(crate) fn offer_details(output: &Path, id: &str) -> Result<PathBuf> {
    use ::graphql_client::GraphQLQuery;
    execute(
        output,
        "offer_details",
        serde_json::to_value(OfferDetails::build_query(offer_details::Variables {
            id: id.into(),
        }))?,
    )
}
