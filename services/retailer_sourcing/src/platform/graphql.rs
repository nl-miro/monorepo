//! Two fixed Shopify operations per storefront, executed independently by three clients.
//! Output roots contain a retailer folder, then implementation and operation folders.
use serde_json::Value;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Storefront used by the fixed GraphQL operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Storefront {
    AnkerAu,
    AnkerCa,
}

impl Storefront {
    pub fn slug(self) -> &'static str {
        match self {
            Self::AnkerAu => "ankerau",
            Self::AnkerCa => "ankerca",
        }
    }
    pub fn country(self) -> &'static str {
        match self {
            Self::AnkerAu => "AU",
            Self::AnkerCa => "CA",
        }
    }
    pub fn currency(self) -> &'static str {
        match self {
            Self::AnkerAu => "AUD",
            Self::AnkerCa => "CAD",
        }
    }
    fn language(self) -> &'static str {
        "EN"
    }
    fn endpoint(self) -> &'static str {
        match self {
            Self::AnkerAu => "https://oneanker-au.myshopify.com/api/2026-10/graphql.json",
            Self::AnkerCa => "https://anker-canada.myshopify.com/api/2026-10/graphql.json",
        }
    }
    // Public browser tokens from Anker's storefront configuration; also used in Bruno.
    fn token(self) -> &'static str {
        match self {
            Self::AnkerAu => "ca89549ed256ec3f2c3a12d113b9f29e",
            Self::AnkerCa => "0a32b45f34ceb286e6757f7b8e71b4b5",
        }
    }
}

#[path = "graphql/custom.rs"]
mod custom;
#[path = "graphql/cynic.rs"]
mod cynic;
#[path = "graphql/graphql_client.rs"]
mod graphql_client;
#[cfg(test)]
#[path = "graphql/operations_tests.rs"]
mod operations_tests;

fn save(path: &Path, value: &Value) -> Result<()> {
    std::fs::create_dir_all(path.parent().ok_or("missing parent")?)?;
    let temporary = path.with_extension("json.new");
    std::fs::write(&temporary, serde_json::to_vec_pretty(value)?)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

fn save_response(path: &Path, body: &[u8]) -> Result<()> {
    match serde_json::from_slice::<Value>(body) {
        Ok(value) => save(path, &value),
        Err(_) => Ok(std::fs::write(path, body)?),
    }
}

/// Runs the catalog offers query once and saves the custom request/response pair.
pub fn catalog_offers_custom(
    storefront: Storefront,
    output: &std::path::Path,
) -> Result<std::path::PathBuf> {
    custom::catalog_offers(storefront, &output.join(storefront.slug()))
}
/// Runs the offer details query once for a Shopify product GID using the custom client.
pub fn offer_details_custom(
    storefront: Storefront,
    output: &std::path::Path,
    product_id: &str,
) -> Result<std::path::PathBuf> {
    custom::offer_details(storefront, &output.join(storefront.slug()), product_id)
}
/// Runs the catalog offers query once using graphql_client.
pub fn catalog_offers_graphql_client(
    storefront: Storefront,
    output: &std::path::Path,
) -> Result<std::path::PathBuf> {
    graphql_client::catalog_offers(storefront, &output.join(storefront.slug()))
}
/// Runs the offer details query once for a Shopify product GID using graphql_client.
pub fn offer_details_graphql_client(
    storefront: Storefront,
    output: &std::path::Path,
    product_id: &str,
) -> Result<std::path::PathBuf> {
    graphql_client::offer_details(storefront, &output.join(storefront.slug()), product_id)
}
/// Runs the catalog offers query once using Cynic.
pub fn catalog_offers_cynic(
    storefront: Storefront,
    output: &std::path::Path,
) -> Result<std::path::PathBuf> {
    cynic::catalog_offers(storefront, &output.join(storefront.slug()))
}
/// Runs the offer details query once for a Shopify product GID using Cynic.
pub fn offer_details_cynic(
    storefront: Storefront,
    output: &std::path::Path,
    product_id: &str,
) -> Result<std::path::PathBuf> {
    cynic::offer_details(storefront, &output.join(storefront.slug()), product_id)
}
