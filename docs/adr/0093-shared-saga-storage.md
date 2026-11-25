# 93. Shared saga storage in retailer sourcing

Status: Accepted

Retailer sourcing stores retailer saga instances through one `SagaRepository`
in the PostgreSQL `sagas` table. Each saga struct implements `Saga` and supplies
a stable type name and a JSON object representing its state. `SagaRecord<S>`
pairs that state with a run ID and retailer code.

The primary key is the saga type and run ID. A typed lookup also checks the
retailer before decoding the payload. Different types can reuse a run ID and
keep independent state. Saving a second record with the same key fails.

The migration labels existing Shopify GraphQL rows with their saga type and
preserves their IDs, retailers and payloads. Rolling back requires every row
to be representable in the former Shopify table.
