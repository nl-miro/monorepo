//! Two fixed Shopify operations executed independently by three clients.
use serde_json::Value;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Public browser token from Anker AU's storefront; matches Bruno's offer details header.
const STOREFRONT_TOKEN: &str = "ca89549ed256ec3f2c3a12d113b9f29e";

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
pub fn catalog_offers_custom(output: &std::path::Path) -> Result<std::path::PathBuf> {
    custom::catalog_offers(output)
}
/// Runs the offer details query once for a Shopify product GID using the custom client.
pub fn offer_details_custom(
    output: &std::path::Path,
    product_id: &str,
) -> Result<std::path::PathBuf> {
    custom::offer_details(output, product_id)
}
/// Runs the catalog offers query once using graphql_client.
pub fn catalog_offers_graphql_client(output: &std::path::Path) -> Result<std::path::PathBuf> {
    graphql_client::catalog_offers(output)
}
/// Runs the offer details query once for a Shopify product GID using graphql_client.
pub fn offer_details_graphql_client(
    output: &std::path::Path,
    product_id: &str,
) -> Result<std::path::PathBuf> {
    graphql_client::offer_details(output, product_id)
}
/// Runs the catalog offers query once using Cynic.
pub fn catalog_offers_cynic(output: &std::path::Path) -> Result<std::path::PathBuf> {
    cynic::catalog_offers(output)
}
/// Runs the offer details query once for a Shopify product GID using Cynic.
pub fn offer_details_cynic(
    output: &std::path::Path,
    product_id: &str,
) -> Result<std::path::PathBuf> {
    cynic::offer_details(output, product_id)
}
