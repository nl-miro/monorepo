ALTER TABLE shopify_graphql_initial_fills
    ADD COLUMN correlation_id uuid,
    ADD COLUMN saga_type text,
    ADD COLUMN payload jsonb;

UPDATE shopify_graphql_initial_fills
SET correlation_id = saga_id, saga_type = 'ShopifyGraphQlInitialFill', payload = '{}'::jsonb;

ALTER TABLE shopify_graphql_initial_fills
    ALTER COLUMN correlation_id SET NOT NULL,
    ALTER COLUMN saga_type SET NOT NULL,
    ALTER COLUMN payload SET NOT NULL,
    ADD CHECK (
        (get_byte(uuid_send(correlation_id), 6) >> 4) = 7 AND
        (get_byte(uuid_send(correlation_id), 8) >> 6) = 2
    );
