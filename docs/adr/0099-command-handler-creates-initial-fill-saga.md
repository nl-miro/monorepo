# 99. Create the initial-fill saga in the command handler

Status: Accepted for implementation

This decision supersedes [ADR 97](0097-saga-initialization-from-envelope-correlation.md)
for ownership of saga creation. The current code still creates the saga when
the listener receives `RetailerFillInitialized`.

The `InitializeRetailerFill` handler will receive the command’s envelope
correlation and a `SagaRepository`. For a new fill it validates the storefront,
creates the Shopify GraphQL initial-fill saga, and persists its initial state,
including the requested contexts, token decision, and storefront URL. Once
the repository transaction commits, the handler returns one
`RetailerFillInitialized` event containing the retailer code. A failed save
returns an error and produces no initialization event.

If a worker retries after the save committed but before the event was recorded,
the handler recovers the same saga by type and correlation, verifies the retailer,
and returns the event using the saved snapshot. The repository’s uniqueness rule
preserves the original saga ID across retries. [ADR 96](0096-saga-record-identities-and-transactions.md)

On event delivery the listener loads the persisted saga and asks it to select
the first token or catalog commands. A missing saga is an error because the
event now confirms persistence. Repeated delivery uses the existing stable
command IDs. [ADR 98](0098-saga-progress-replay-and-failure-observation.md)
