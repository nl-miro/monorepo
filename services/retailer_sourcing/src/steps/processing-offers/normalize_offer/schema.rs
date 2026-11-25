// @generated automatically by Diesel CLI.

use diesel::table;

table! {
    offer_details_normalized (id) {
        id -> Uuid,
        offer_id -> Uuid,
        normalized_at -> Timestamptz,
        storage -> Jsonb,
    }
}
