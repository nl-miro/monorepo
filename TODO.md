# TODO

Status follows the [TODO workflow](AGENTS.md#todo-workflow):

| Marker | Phase                      |
| ------ | -------------------------- |
| `[ ]`  | Pending                    |
| `[.]`  | Planning in progress       |
| `[>]`  | Planning complete          |
| `[~]`  | Implementation in progress |
| `[x]`  | Implemented                |

Flow: `[ ]` → `[.]` → `[>]` → `[~]` → `[x]`.

- [x] Move the three Shopify GraphQL initial-fill steps into the root `steps/graphql` module.

- [x] Move sitemap retrieval, processing and consolidation into `steps/sitemap` and retire the HTTP retrieval endpoint.

Shopify GraphQL initial-fill saga through structuralized offer details, in the
[implementation plan](docs/plans/saga-implementation.md)'s delivery order:

- [x] Saga storage: consolidate the unshipped migrations into one, and add the saga record with its payload state, uniqueness, PostgreSQL 18 UUID checks, and locked transaction-scoped transitions. Move CI to PostgreSQL 18.

- [x] Initialization: move saga creation into the `RetailerFillInitialized` listener, simplify the handler and event, and remove duplicated identity from the saga payload.

- [x] Step contracts and command identity: expose reusable step contracts and context-keyed builders that use the persisted context snapshot and the internal saga ID namespace. Remove payload and `KeyedCommand` `run_id`, and move metadata consumers to envelope correlation.

- [x] Transitions and progress: implement transitions, canonical results, payload progress state, and replayed dispatch. Register the saga listeners and remove duplicate dispatch owners.

- [x] Failure reconciliation: reconcile exhausted commands and saga event deliveries into terminal outcomes, and evaluate the fill against the Success Criteria.

- [x] End-to-end verification: complete queue and migration verification, then run a token and a tokenless retailer through the API to their final outcomes and inspect the structured artifacts.

## Saga review fixes

- [x] Subscriber order and import failure isolation.
- [x] Start validation, correlation receipts and blocking dispatch.
- [x] Coordinated stale-reservation recovery.
- [x] Per-saga reconciliation failure isolation.
- [x] Queue correlation indexes and unchanged-write avoidance.
- [x] Expected-work matching before retailer validation.
- [x] Consolidated decisions and consistent documentation.

## Saga implementation v3

The [v3 plan](docs/plans/saga-implementation-v3.md) keeps the ADR 91–92 module
layout and redoes the saga work; implementation and any reset have not started.

- [>] Confirm the v3 starting point: module layout kept, no saga work, `make ci` passing.
- [>] Bus: envelope-aware subscriptions, correlation and causation in reactions, `KeyedCommand` without `run_id`, batch observers, one batch path.
- [>] Storage: versioned JSON payload, saga and queue evidence index migrations, `SagaStatus`, and a repository that saves only changed records.
- [>] Steps: `InitializeRetailerFill` with shared validation, step payloads without `run_id`, and builders taking the saga namespace and context snapshot.
- [>] Saga: payload model, catalog-proportional matching, transitions, shared registration with the saga first, and the `202` start endpoint.
- [>] Reconciliation: pending-ID evidence queries, queue failure rules, per-saga isolation and throttled logs, serialized passes, startup pass and batch observation.
- [>] Recovery: async `MANUAL_BATCH_CONCURRENCY` admission (default 4), advisory locks, one connection reused per batch, reconciliation after post-sweep errors, and deployment rules; remove `COMMAND_WORKERS`.
- [>] Cleanup: move tests next to their code and delete `src/v2/application`.
- [>] Documentation and verification: final ADRs, README, Bruno requests, limits/cancellation/panic/two-process tests, query plans, 100/1,000/10,000-product measurements, PostgreSQL suite and both live fills.
