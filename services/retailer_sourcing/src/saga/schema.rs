use diesel::table;

table! {
    sagas (id) {
        saga_type -> Text,
        id -> Uuid,
        correlation_id -> Uuid,
        status -> Text,
        retailer_code -> Text,
        payload -> Jsonb,
    }
}
