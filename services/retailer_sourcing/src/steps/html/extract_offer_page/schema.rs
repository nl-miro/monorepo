// @generated automatically by Diesel CLI.

use diesel::table;

table! {
    offer_details_extracted (id) {
        id -> Uuid,
        offer_id -> Uuid,
        extracted_at -> Timestamptz,
        storage -> Jsonb,
    }
}
