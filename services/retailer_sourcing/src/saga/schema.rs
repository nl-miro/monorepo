use diesel::table;

table! {
    shopify_graphql_initial_fills (saga_id) {
        saga_id -> Uuid,
        correlation_id -> Uuid,
        saga_type -> Text,
        payload -> Jsonb,
    }
}
