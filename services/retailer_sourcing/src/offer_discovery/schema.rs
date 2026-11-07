// @generated automatically by Diesel CLI.

diesel::table! {
    offer_details_extracted (id) {
        id -> Uuid,
        offer_id -> Uuid,
        extracted_at -> Timestamptz,
        storage -> Jsonb,
    }
}

diesel::table! {
    offer_details_normalized (id) {
        id -> Uuid,
        offer_id -> Uuid,
        normalized_at -> Timestamptz,
        storage -> Jsonb,
    }
}

diesel::table! {
    offer_details_raw (id) {
        id -> Uuid,
        offer_id -> Uuid,
        url -> Text,
        fetched_at -> Timestamptz,
        retailer_code -> Text,
        storage -> Jsonb,
    }
}

diesel::table! {
    offer_details_structuralized (id) {
        id -> Uuid,
        offer_id -> Uuid,
        structuralized_at -> Timestamptz,
        storage -> Jsonb,
    }
}

diesel::table! {
    offer_export (id) {
        id -> Uuid,
        offer_id -> Uuid,
        source_object_id -> Uuid,
        source -> Jsonb,
        stats -> Jsonb,
        status -> Text,
        prepared_body -> Nullable<Text>,
        storage -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    offer_details_extracted,
    offer_details_normalized,
    offer_details_raw,
    offer_details_structuralized,
    offer_export,
);
