// @generated automatically by Diesel CLI.

use diesel::table;

table! {
    sitemaps_processed (id) {
        id -> Uuid,
        retrieval_id -> Uuid,
        retailer_code -> Text,
        url_count -> Int4,
        processed_at -> Timestamptz,
        storage -> Jsonb,
    }
}
