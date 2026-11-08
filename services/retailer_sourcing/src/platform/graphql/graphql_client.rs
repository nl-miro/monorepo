//! Execute each review query once and save its request and response as pretty JSON.
use super::{Result, Storefront, save, save_response};
use serde_json::Value;
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
    let folder = output.join("graphql_client").join(name);
    save(&folder.join("request.json"), &request)?;
    let response = http_request(storefront, &request, storefront_token)?.send()?;
    let status = response.status();
    save_response(&folder.join("response.json"), &response.bytes()?)?;
    if !status.is_success() {
        return Err(format!("HTTP {status}; response saved in {}", folder.display()).into());
    }
    Ok(folder)
}
mod ankerau {
    #[allow(clippy::upper_case_acronyms)]
    type HTML = String;
    #[allow(clippy::upper_case_acronyms)]
    type URL = String;
    type DateTime = String;
    type Decimal = String;
    type Color = String;

    #[derive(::graphql_client::GraphQLQuery)]
    #[graphql(
        schema_path = "src/platform/graphql/ankerau/schema.graphql",
        query_path = "src/platform/graphql/ankerau/catalog_offers.graphql",
        response_derives = "Debug"
    )]
    pub(super) struct CatalogOffers;
    #[derive(::graphql_client::GraphQLQuery)]
    #[graphql(
        schema_path = "src/platform/graphql/ankerau/schema.graphql",
        query_path = "src/platform/graphql/ankerau/offer_details.graphql",
        response_derives = "Debug"
    )]
    pub(super) struct OfferDetails;
}
mod ankerca {
    #[allow(clippy::upper_case_acronyms)]
    type HTML = String;
    #[allow(clippy::upper_case_acronyms)]
    type URL = String;
    type DateTime = String;
    type Decimal = String;
    type Color = String;

    #[derive(::graphql_client::GraphQLQuery)]
    #[graphql(
        schema_path = "src/platform/graphql/ankerca/schema.graphql",
        query_path = "src/platform/graphql/ankerca/catalog_offers.graphql",
        response_derives = "Debug"
    )]
    pub(super) struct CatalogOffers;
    #[derive(::graphql_client::GraphQLQuery)]
    #[graphql(
        schema_path = "src/platform/graphql/ankerca/schema.graphql",
        query_path = "src/platform/graphql/ankerca/offer_details.graphql",
        response_derives = "Debug"
    )]
    pub(super) struct OfferDetails;
}
pub(super) fn catalog_request(storefront: Storefront) -> Result<Value> {
    use ::graphql_client::GraphQLQuery;
    match storefront {
        Storefront::AnkerAu => Ok(serde_json::to_value(ankerau::CatalogOffers::build_query(
            ankerau::catalog_offers::Variables { after: None },
        ))?),
        Storefront::AnkerCa => Ok(serde_json::to_value(ankerca::CatalogOffers::build_query(
            ankerca::catalog_offers::Variables { after: None },
        ))?),
    }
}
pub(super) fn details_request(storefront: Storefront, id: &str) -> Result<Value> {
    use ::graphql_client::GraphQLQuery;
    match storefront {
        Storefront::AnkerAu => Ok(serde_json::to_value(ankerau::OfferDetails::build_query(
            ankerau::offer_details::Variables { id: id.into() },
        ))?),
        Storefront::AnkerCa => Ok(serde_json::to_value(ankerca::OfferDetails::build_query(
            ankerca::offer_details::Variables { id: id.into() },
        ))?),
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
