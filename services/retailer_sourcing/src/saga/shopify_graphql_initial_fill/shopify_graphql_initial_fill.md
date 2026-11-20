# Shopify GraphQL initial fill

1. `POST /api/initial-offer-fills` dispatches `InitializeRetailerFill` per retailer
   with the original flow correlation in envelope metadata.
2. The handler validates support and nonempty contexts, then emits the
   retailer-only `RetailerFillInitialized`. It does not access saga storage.
3. The listener creates or recovers a saga by type and envelope correlation.
   It snapshots contexts, token policy and storefront URL, then returns the
   token command or context-specific catalog commands after committing.
4. Token results retain the first acquired token and dispatch the saved contexts.
   Catalog results retain the first complete product set and resolved context.
   Each product/context gets a stable detail command keyed by requested context.
5. Fetched and extracted results retain their first event/object references and
   dispatch extraction and structuralization using those canonical results.
   Repeated results return the same commands even after further progress.
6. Every transition locks the whole saga row and saves payload and fill status
   together. All listener routing uses envelope correlation; the saga record ID
   only supplies stable command namespaces. Step payloads contain no run ID.
7. Startup and manual command/event batches reconcile exhausted queue work.
   Pending success events protect their commands from premature failure. Missing
   commands after exhausted dispatch fail their blocked work. Workers remain
   disabled; keep driving both queue endpoints until the saga is terminal.
8. A fill is `succeeded` only when all requested catalogs succeed and every
   discovered product/context is structuralized. Any failure or evidenced skip
   makes a terminal fill `failed`. Empty catalogs succeed. Import preparation
   subscribes independently to structuralized events.

Retain saga state, queue rows, canonical events and referenced artifacts
indefinitely. Replay depends on those records. Tokens are replayed unchanged;
refresh and expiry handling are outside this phase. Start a new fill to retry a
terminal failure.

Inspect `sagas.status` and `sagas.payload` for progress. The payload's catalog and
product maps identify pending work through missing result references, completed
steps through canonical references, and terminal failures through their reason.
Development artifact listings expose envelope `correlation_id`.

See [ADR 98](../../../../../docs/adr/0098-saga-progress-replay-and-failure-observation.md).
Run `make test-postgres` for controlled pipeline/failure scenarios and
`make test-saga-live` for complete API-driven Anker AU and Ugreen EU verification.
