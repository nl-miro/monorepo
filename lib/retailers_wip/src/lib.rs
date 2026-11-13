//! Retailer inventory and local initial-fill context contracts.

pub mod graphql_errors;
pub mod initial_fill;
pub mod retailers;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailers_wip");
    }
}
