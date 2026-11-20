// @generated automatically by Diesel CLI.

use diesel::table;

table! {
    offer_details_raw (id) {
        id -> Uuid,
        offer_id -> Uuid,
        url -> Text,
        fetched_at -> Timestamptz,
        retailer_code -> Text,
        storage -> Jsonb,
    }
}
