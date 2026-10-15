// @generated automatically by Diesel CLI.

diesel::table! {
    offer_details_extracted (id) {
        id -> Uuid,
        offer_id -> Uuid,
        extracted_at -> Timestamptz,
    }
}

diesel::table! {
    extracted_offers_data (id) {
        id -> Uuid,
        document -> Jsonb,
    }
}

diesel::table! {
    offer_details_normalized (id) {
        id -> Uuid,
        offer_id -> Uuid,
        normalized_at -> Timestamptz,
    }
}

diesel::table! {
    normalized_offers_data (id) {
        id -> Uuid,
        document -> Jsonb,
    }
}

diesel::table! {
    offer_details_raw (id) {
        id -> Uuid,
        offer_id -> Uuid,
        url -> Text,
        body_size -> Int4,
        fetched_at -> Timestamptz,
        retailer_code -> Text,
    }
}

diesel::table! {
    raw_offers_data (id) {
        id -> Uuid,
        body -> Text,
    }
}

diesel::table! {
    offer_details_structuralized (id) {
        id -> Uuid,
        offer_id -> Uuid,
        structuralized_at -> Timestamptz,
    }
}

diesel::table! {
    structuralized_offers_data (id) {
        id -> Uuid,
        document -> Jsonb,
    }
}

diesel::joinable!(extracted_offers_data -> offer_details_extracted (id));
diesel::joinable!(normalized_offers_data -> offer_details_normalized (id));
diesel::joinable!(raw_offers_data -> offer_details_raw (id));
diesel::joinable!(structuralized_offers_data -> offer_details_structuralized (id));

diesel::allow_tables_to_appear_in_same_query!(
    offer_details_extracted,
    extracted_offers_data,
    offer_details_normalized,
    normalized_offers_data,
    offer_details_raw,
    raw_offers_data,
    offer_details_structuralized,
    structuralized_offers_data,
);
