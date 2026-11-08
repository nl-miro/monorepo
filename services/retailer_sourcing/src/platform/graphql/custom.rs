//! Execute each review query once and save its request and response as pretty JSON.
use super::{Result, STOREFRONT_TOKEN, save, save_response};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

fn execute(
    output: &Path,
    name: &str,
    request: Value,
    storefront_token: Option<&str>,
) -> Result<PathBuf> {
    let folder = output.join("custom").join(name);
    save(&folder.join("request.json"), &request)?;
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .build()?
        .post("https://oneanker-au.myshopify.com/api/2026-10/graphql.json")
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&request)?);
    if let Some(token) = storefront_token {
        builder = builder.header("X-Shopify-Storefront-Access-Token", token);
    }
    let response = builder.send()?;
    let status = response.status();
    save_response(&folder.join("response.json"), &response.bytes()?)?;
    if !status.is_success() {
        return Err(format!("HTTP {status}; response saved in {}", folder.display()).into());
    }
    Ok(folder)
}

pub(crate) fn catalog_offers(output: &Path) -> Result<PathBuf> {
    execute(
        output,
        "catalog_offers",
        json!({
            "query":include_str!("ankerau/catalog_offers.graphql"), "operationName":"CatalogOffers", "variables":{"after":null}
        }),
        None,
    )
}
pub(crate) fn offer_details(output: &Path, id: &str) -> Result<PathBuf> {
    execute(
        output,
        "offer_details",
        json!({
            "query":include_str!("ankerau/offer_details.graphql"), "operationName":"OfferDetails", "variables":{"id":id}
        }),
        Some(STOREFRONT_TOKEN),
    )
}
