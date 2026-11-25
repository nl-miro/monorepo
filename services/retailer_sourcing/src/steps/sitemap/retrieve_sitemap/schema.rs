// @generated automatically by Diesel CLI.

use diesel::table;

table! {
    sitemaps_raw (id) {
        id -> Uuid,
        retrieval_id -> Uuid,
        url -> Text,
        last_modified -> Nullable<Timestamptz>,
        fetched_at -> Timestamptz,
        storage -> Jsonb,
    }
}
