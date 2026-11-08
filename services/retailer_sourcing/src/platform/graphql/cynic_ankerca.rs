#[path = "ankerca_cynic_catalog_offers.rs"]
pub(super) mod catalog_offers;
#[path = "ankerca_cynic_offer_details.rs"]
pub(super) mod offer_details;
mod schema {
    cynic::use_schema!("src/platform/graphql/ankerca/schema.graphql");
}
