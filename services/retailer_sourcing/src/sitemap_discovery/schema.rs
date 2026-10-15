// @generated automatically by Diesel CLI.

diesel::table! {
    sitemaps_consolidated (id) {
        id -> Uuid,
        consolidated_at -> Timestamptz,
        stats -> Jsonb,
        references -> Jsonb,
        storage -> Jsonb,
    }
}

diesel::table! {
    consolidated_sitemaps_data (id) {
        id -> Uuid,
        content -> Jsonb,
    }
}

diesel::table! {
    sitemaps_processed (id) {
        id -> Uuid,
        retrieval_id -> Uuid,
        retailer_code -> Text,
        document_size -> Int4,
        url_count -> Int4,
        processed_at -> Timestamptz,
    }
}

diesel::table! {
    processed_sitemaps_data (id) {
        id -> Uuid,
        document -> Jsonb,
    }
}

diesel::table! {
    sitemaps_raw (id) {
        id -> Uuid,
        retrieval_id -> Uuid,
        url -> Text,
        last_modified -> Nullable<Timestamptz>,
        body_size -> Int4,
        fetched_at -> Timestamptz,
    }
}

diesel::table! {
    raw_sitemap_documents_data (id) {
        id -> Uuid,
        body -> Text,
    }
}

diesel::joinable!(consolidated_sitemaps_data -> sitemaps_consolidated (id));
diesel::joinable!(processed_sitemaps_data -> sitemaps_processed (id));
diesel::joinable!(raw_sitemap_documents_data -> sitemaps_raw (id));

diesel::allow_tables_to_appear_in_same_query!(
    sitemaps_consolidated,
    consolidated_sitemaps_data,
    sitemaps_processed,
    processed_sitemaps_data,
    sitemaps_raw,
    raw_sitemap_documents_data,
);
