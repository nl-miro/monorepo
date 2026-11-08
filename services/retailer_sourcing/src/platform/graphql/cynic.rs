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
    let folder = output.join("cynic").join(name);
    save(&folder.join("request.json"), &request)?;
    let response = http_request(storefront, &request, storefront_token)?.send()?;
    let status = response.status();
    save_response(&folder.join("response.json"), &response.bytes()?)?;
    if !status.is_success() {
        return Err(format!("HTTP {status}; response saved in {}", folder.display()).into());
    }
    Ok(folder)
}

#[path = "cynic_ankerau.rs"]
mod ankerau;
#[path = "cynic_ankerca.rs"]
mod ankerca;

fn request<Response, Variables: serde::Serialize>(
    storefront: Storefront,
    mut operation: cynic::Operation<Response, Variables>,
) -> Result<Value> {
    let opening = operation
        .query
        .find('{')
        .ok_or("missing operation selection")?;
    operation.query.insert_str(
        opening,
        &format!(
            "@inContext(country: {}, language: {}) ",
            storefront.country(),
            storefront.language()
        ),
    );
    Ok(
        json!({"query":operation.query,"operationName":operation.operation_name,"variables":operation.variables}),
    )
}
pub(super) fn catalog_request(storefront: Storefront) -> Result<Value> {
    use cynic::QueryBuilder;
    match storefront {
        Storefront::AnkerAu => request(
            storefront,
            ankerau::catalog_offers::CatalogOffers::build(
                ankerau::catalog_offers::CatalogOffersVariables { after: None },
            ),
        ),
        Storefront::AnkerCa => request(
            storefront,
            ankerca::catalog_offers::CatalogOffers::build(
                ankerca::catalog_offers::CatalogOffersVariables { after: None },
            ),
        ),
    }
}
pub(super) fn details_request(storefront: Storefront, id: &str) -> Result<Value> {
    use cynic::QueryBuilder;
    match storefront {
        Storefront::AnkerAu => request(
            storefront,
            ankerau::offer_details::OfferDetails::build(
                ankerau::offer_details::OfferDetailsVariables {
                    id: &cynic::Id::new(id),
                },
            ),
        ),
        Storefront::AnkerCa => request(
            storefront,
            ankerca::offer_details::OfferDetails::build(
                ankerca::offer_details::OfferDetailsVariables {
                    id: &cynic::Id::new(id),
                },
            ),
        ),
    }
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
