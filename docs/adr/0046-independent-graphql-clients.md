# ADR 46: Independent GraphQL client experiments

Status: Accepted

## Context

We want to compare graphql_client and Cynic with the custom Anker AU downloader,
while retaining discovery of new public fields at runtime.

## Decision

Keep three independent implementations behind separate platform functions.
graphql_client generates typed operations from GraphQL documents; Cynic generates
them from Rust fragments. Both use the existing blocking reqwest version directly.
Compile known catalogue operations against a checked-in Shopify schema, then
independently discover and fetch exhaustive selections at runtime. Match product
IDs and cursors between typed traversal and dynamic enrichment. Preserve request
and response evidence before typed decoding. The existing CLI keeps its default.

## Consequences

Runtime-added fields remain available as raw JSON. Independence duplicates
transport and pagination logic deliberately for this experiment. The alternatives
make extra requests and fail if the catalogue changes between paired page reads.
They are not integrated into the production offer pipeline.
