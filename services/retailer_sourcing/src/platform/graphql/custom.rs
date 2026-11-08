//! Execute each review query once and save its request and response as pretty JSON.
use super::{Result, Storefront, save, save_response};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

pub(super) fn http_request(
    storefront: Storefront,
    request: &Value,
    storefront_token: Option<&str>,
) -> Result<reqwest::blocking::RequestBuilder> {
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .build()?
        .post(storefront.endpoint())
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(request)?);
    if let Some(token) = storefront_token {
        builder = builder.header("X-Shopify-Storefront-Access-Token", token);
    }
    Ok(builder)
}

fn execute(
    storefront: Storefront,
    output: &Path,
    name: &str,
    request: Value,
    storefront_token: Option<&str>,
) -> Result<PathBuf> {
    let folder = output.join("custom").join(name);
    save(&folder.join("request.json"), &request)?;
    let response = http_request(storefront, &request, storefront_token)?.send()?;
    let status = response.status();
    save_response(&folder.join("response.json"), &response.bytes()?)?;
    if !status.is_success() {
        return Err(format!("HTTP {status}; response saved in {}", folder.display()).into());
    }
    Ok(folder)
}

pub(super) fn catalog_request(storefront: Storefront) -> Result<Value> {
    let query = match storefront {
        Storefront::AnkerAu => include_str!("ankerau/catalog_offers.graphql"),
        Storefront::AnkerCa => include_str!("ankerca/catalog_offers.graphql"),
    };
    Ok(json!({"query":query, "operationName":"CatalogOffers", "variables":{"after":null}}))
}
pub(super) fn details_request(storefront: Storefront, id: &str) -> Result<Value> {
    let query = match storefront {
        Storefront::AnkerAu => include_str!("ankerau/offer_details.graphql"),
        Storefront::AnkerCa => include_str!("ankerca/offer_details.graphql"),
    };
    Ok(json!({"query":query, "operationName":"OfferDetails", "variables":{"id":id}}))
}

pub(crate) fn catalog_offers(storefront: Storefront, output: &Path) -> Result<PathBuf> {
    execute(
        storefront,
        output,
        "catalog_offers",
        catalog_request(storefront)?,
        None,
    )
}
pub(crate) fn offer_details(storefront: Storefront, output: &Path, id: &str) -> Result<PathBuf> {
    execute(
        storefront,
        output,
        "offer_details",
        details_request(storefront, id)?,
        Some(storefront.token()),
    )
}
