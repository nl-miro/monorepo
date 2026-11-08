//! Outbound GraphQL catalogue downloads.

#[cfg(test)]
#[path = "graphql/operations_tests.rs"]
mod operations_tests;

#[path = "graphql/custom.rs"]
mod custom;

/// Downloads the public AU/EN catalogue, retaining separate evidence for every attempt.
pub fn download_ankerau(
    output_dir: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    custom::download_ankerau(output_dir)
}

#[path = "graphql/graphql_client.rs"]
mod graphql_client;
/// Downloads Anker AU using graphql_client for typed catalogue traversal.
pub fn download_ankerau_graphql_client(
    output_dir: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    graphql_client::download_ankerau(output_dir)
}

#[path = "graphql/cynic.rs"]
mod cynic;
/// Downloads Anker AU using cynic for typed catalogue traversal.
pub fn download_ankerau_cynic(
    output_dir: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    cynic::download_ankerau(output_dir)
}

/// Downloads at most `limit` products in Shopify ID order, including nested connections.
pub fn download_ankerau_limited(
    output_dir: &std::path::Path,
    limit: usize,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    custom::download_ankerau_limited(output_dir, Some(limit))
}

/// Downloads at most `limit` products in Shopify ID order, including nested connections.
pub fn download_ankerau_graphql_client_limited(
    output_dir: &std::path::Path,
    limit: usize,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    graphql_client::download_ankerau_limited(output_dir, Some(limit))
}

/// Downloads at most `limit` products in Shopify ID order, including nested connections.
pub fn download_ankerau_cynic_limited(
    output_dir: &std::path::Path,
    limit: usize,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    cynic::download_ankerau_limited(output_dir, Some(limit))
}

/// Runs the catalog offers query once and saves the custom request/response pair.
pub fn catalog_offers_custom(
    output: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    custom::operations::catalog_offers(output)
}
/// Runs the offer details query once for a Shopify product GID using the custom client.
pub fn offer_details_custom(
    output: &std::path::Path,
    product_id: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    custom::operations::offer_details(output, product_id)
}
/// Runs the catalog offers query once using graphql_client.
pub fn catalog_offers_graphql_client(
    output: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    graphql_client::operations::catalog_offers(output)
}
/// Runs the offer details query once for a Shopify product GID using graphql_client.
pub fn offer_details_graphql_client(
    output: &std::path::Path,
    product_id: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    graphql_client::operations::offer_details(output, product_id)
}
/// Runs the catalog offers query once using Cynic.
pub fn catalog_offers_cynic(
    output: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    cynic::operations::catalog_offers(output)
}
/// Runs the offer details query once for a Shopify product GID using Cynic.
pub fn offer_details_cynic(
    output: &std::path::Path,
    product_id: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    cynic::operations::offer_details(output, product_id)
}
