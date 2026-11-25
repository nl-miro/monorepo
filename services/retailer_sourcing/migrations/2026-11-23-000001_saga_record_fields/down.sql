ALTER TABLE shopify_graphql_initial_fills
    DROP COLUMN payload,
    DROP COLUMN saga_type,
    DROP COLUMN correlation_id;
