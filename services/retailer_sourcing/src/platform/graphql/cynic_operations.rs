//! Execute each review query once and save its request and unmodified response.
use super::{Result, save};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

fn execute(output: &Path, name: &str, request: Value) -> Result<PathBuf> {
    let folder = output.join("cynic").join(name);
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

#[path = "cynic_catalog_offers.rs"]
mod catalog_offers;
#[path = "cynic_offer_details.rs"]
mod offer_details;

fn request<Response, Variables: serde::Serialize>(
    mut operation: cynic::Operation<Response, Variables>,
) -> Result<Value> {
    let opening = operation
        .query
        .find('{')
        .ok_or("missing operation selection")?;
    operation
        .query
        .insert_str(opening, "@inContext(country: AU, language: EN) ");
    Ok(
        json!({"query":operation.query,"operationName":operation.operation_name,"variables":operation.variables}),
    )
}
pub(crate) fn catalog_offers(output: &Path) -> Result<PathBuf> {
    use cynic::QueryBuilder;
    execute(
        output,
        "catalog_offers",
        request(catalog_offers::CatalogOffers::build(
            catalog_offers::CatalogOffersVariables { after: None },
        ))?,
    )
}
pub(crate) fn offer_details(output: &Path, id: &str) -> Result<PathBuf> {
    use cynic::QueryBuilder;
    execute(
        output,
        "offer_details",
        request(offer_details::OfferDetails::build(
            offer_details::OfferDetailsVariables {
                id: &cynic::Id::new(id),
            },
        ))?,
    )
}
