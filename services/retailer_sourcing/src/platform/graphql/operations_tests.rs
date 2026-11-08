//! Keep the fixed request documents aligned with their Bruno originals.
use serde_json::Value;

#[test]
fn fixed_queries_match_bruno() {
    for (bruno, document, original_name, name) in [
        (
            include_str!("../../../../../bruno/ankerau-catalog/08-availability-prices.bru"),
            include_str!("catalog_offers.graphql"),
            "CatalogAvailabilityPrices",
            "CatalogOffers",
        ),
        (
            include_str!("../../../../../bruno/ankerau-catalog/single.bru"),
            include_str!("offer_details.graphql"),
            "Single",
            "OfferDetails",
        ),
    ] {
        let body = bruno
            .split("body:json {\n")
            .nth(1)
            .unwrap()
            .rsplit_once("\n}")
            .unwrap()
            .0;
        let request: Value = serde_json::from_str(body).unwrap();
        assert_eq!(
            document.trim(),
            request["query"]
                .as_str()
                .unwrap()
                .replace(original_name, name)
        );
    }
}
