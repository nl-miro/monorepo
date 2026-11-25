CREATE TABLE shopify_graphql_initial_fills (
    saga_id uuid PRIMARY KEY CHECK (
        (get_byte(uuid_send(saga_id), 6) >> 4) = 7 AND
        (get_byte(uuid_send(saga_id), 8) >> 6) = 2
    )
);
