#[allow(unused_imports)]
pub use crate::graphql::nordic::{ENDPOINT, raw};
#[allow(non_upper_case_globals)]
pub const Store: crate::graphql::nordic::Store = crate::graphql::nordic::Store {
    country: "FI",
    currency: money::Currency::EUR,
};
#[cfg(test)]
use serde_json;
#[cfg(test)]
mod source_tests {
    use super::*;
    #[test]
    fn local_fixture_and_query_match_the_retailer() {
        let body = include_str!("response.json");
        let _: raw::Response = serde_json::from_str(body).unwrap();
        Store
            .validate_download("https://www.ankernordics.com/products/t90r0121", body)
            .unwrap();
        let request: serde_json::Value = serde_json::from_str(
            &Store
                .request("https://www.ankernordics.com/products/t90r0121")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            request["query"],
            include_str!("../../../../graphql/nordic/product.graphql")
        );
        assert_eq!(request["variables"]["country"], Store.country());
    }
}
