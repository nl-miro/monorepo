//! Workspace for future experimental retailer integrations.

pub mod retailers;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailers_wip");
    }
}
