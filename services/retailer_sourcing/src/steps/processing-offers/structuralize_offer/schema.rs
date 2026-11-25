// @generated automatically by Diesel CLI.

use diesel::table;

table! {
    offer_details_structuralized (id) {
        id -> Uuid,
        offer_id -> Uuid,
        structuralized_at -> Timestamptz,
        storage -> Jsonb,
    }
}
