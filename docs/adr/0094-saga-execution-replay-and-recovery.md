# 94. Saga progress, replay and failure observation

Status: Accepted

Supersedes the initial-fill reaction ownership and payload identity described in
[ADR 59](0059-multi-market-shopify-initial-fills.md) and
[ADR 74](0074-drop-market-discovery-stage.md). Completes the initialization
boundary in the initialization event contract.

## Decision

The Shopify GraphQL saga owns dispatch from initialization through
structuralization. Step modules register reusable handlers; import preparation
remains an independent subscriber. Every saga listener locates the record by
saga type and envelope correlation, validates the retailer, and matches the
expected command's ID against envelope causation. Step payloads and the keyed
command wrapper contain no `run_id` or `saga_id`.

Every transition takes the saga row lock, updates its JSON payload and status,
and commits before returning commands. The payload retains the requested
contexts, token decision, storefront URL and acquired token. Catalog entries
retain their first result event, resolved context and unique product set.
Each product retains accepted fetched, extracted and structuralized event/object
references, or a terminal failure reason. Absent results represent pending work;
the last accepted result identifies the next step. Explicit skips require a
reason and evidence; queue errors never imply disappearance or unpublication.
No separate progress table or per-product command copies are introduced.

The internal saga ID namespaces token, requested-context catalog and
requested-context/product detail commands. Derived command IDs retain their
source-object namespace. Requested identity stays distinct from resolved
context. Replay reconstructs the original commands from the snapshot and
canonical results, even after later transitions. A retried fetch with a new
object/event ID cannot fork the downstream chain. Catalog/token inputs are in
the payload; nested fetched/extracted command inputs come from retained result
events.

Replay support is indefinite. Retain command deduplication rows, triggering
and canonical result events, saga state and referenced artifacts indefinitely.
Deleting completed command rows permits replay to enqueue already executed work;
deleting canonical events prevents command reconstruction. A later retention
policy needs a new decision defining an owned replay horizon or tombstones.
Token expiration, refresh and execution-time token resolution are outside this
phase: replay preserves the acquired token unchanged.

## Failure observation and completion

A queue observer recomputes outstanding command IDs while holding the same saga
lock. It uses the runner's Mulac 0.5 default attempt limits. `Failed` commands or
events at that limit are exhausted; no transition to `Dead` is required.
Reserved and retryable work remains pending. Pending result delivery protects
against a command failing after publication. Accepted outcomes take precedence
over queue failures. An exhausted trigger only fails successors that were never
dispatched; unrelated subscriber failures cannot undo committed saga work.
Failure reasons identify the queue entry and blocked step. Downstream work
blocked by that failure is terminal with its product or catalog.

The executable reconciles on startup. The saga registers a generic bus observer
that runs after manual command/event batches, including batches with item
failures. Observation errors fail startup or return HTTP 500 with a retryable
batch response; details are logged. Command workers remain disabled in this
phase, and events remain manually driven. Enabling background workers requires
adding periodic reconciliation to their lifecycle.

The status is `in_progress` until every catalog and discovered product/context
has a terminal outcome. `succeeded` requires all catalogs to succeed and every
product to reach structuralization. Empty catalogs succeed. Any failure or
explicit skip makes a terminal fill `failed`. Replay never reopens a terminal
fill; start a new fill to retry it. Import delivery is independent of this
structuralization completion boundary.

## Consequences and verification

Whole-payload writes serialize concurrent transitions and retain simple recovery
semantics. They also cause quadratic cumulative write volume as a fill grows.
A local PostgreSQL 18 measurement with 1,000 products and three canonical results
per product used approximately 390 KB, and a locked
catalog replay took 43 ms. These are local measurements, not throughput targets.

Tests cover the registered API pipeline, metadata propagation, canonical replay,
concurrent completion, partial dispatch after commit, exhausted command/event
delivery, pending success, unrelated subscriber failure and terminal skips.
`make test-saga-live` separately runs complete Anker AU and Ugreen EU fills
through the API with direct storefront HTTP, real storage and isolated PostgreSQL
schemas. Concurrent manual batches exercise reservations without enabling workers. These live
checks create immutable object-storage artifacts and require external services.


The 2026-11-20 live verification passed both complete fills: 152 Anker AU
artifacts and 1,234 Ugreen EU artifacts across five contexts. Both persisted
`succeeded`; every artifact passed product, variant, context and pricing checks.
Ugreen also used a second service process to drive concurrent manual HTTP
batches with workers disabled. See the implementation plan for context counts.

## Recovery and delivery isolation

Saga subscribers register before import preparation. An import subscriber failure
must not prevent acceptance of a successfully structuralized product.

Startup and manual batches sweep command and event reservations older than five
minutes using Mulac's six-attempt policy. Concurrent batches hold shared PostgreSQL
advisory locks on dedicated connections. Sweeping requires the exclusive lock and
is deferred while any batch is active. Guards live inside blocking execution,
including after an HTTP client disconnects. Recent abandoned reservations are
recovered on a subsequent manual batch. Workers remain disabled.

Malformed saga payloads are logged individually and counted while healthy sagas
continue. Infrastructure errors still fail startup or the batch. Reconciliation
logs scanned and failed counts. Queue correlation expression indexes support
indefinitely retained history.

The start endpoint validates all requested retailer configurations before dispatch,
then returns HTTP 202 with retailer/correlation receipts. Saga creation remains
asynchronous. Dispatch runs outside Tokio runtime threads. Multi-retailer dispatch
is not atomic; a queue failure can follow earlier successful inserts.
