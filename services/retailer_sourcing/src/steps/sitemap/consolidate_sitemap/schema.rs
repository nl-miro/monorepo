// @generated automatically by Diesel CLI.

use diesel::table;

table! {
    sitemaps_consolidated (id) {
        id -> Uuid,
        consolidated_at -> Timestamptz,
        stats -> Jsonb,
        references -> Jsonb,
        storage -> Jsonb,
    }
}
