# Saga implementation v2

Status: proposed implementation plan, 2026-11-21. Implementation has not started.

This is a standalone plan for rebuilding the Shopify GraphQL initial-fill saga.
It replaces the implementation approach in [v1](saga-implementation.md), while
preserving its identity, sequencing, replay, and strict success contracts. It
incorporates the failure cases and operational costs exposed by implementation
and review. It does not authorize a Git reset, database reset, or deployment.

**Storage decision:** keep all progress in the saga JSON payload, as agreed for
this revision. Persist direct command-ID lookup data, maintain counters, and
serialize only changed state. These changes remove redundant work; they do not
remove whole-payload decoding or quadratic cumulative writes. Accept and measure
that remaining scaling limit in this phase. Separate progress tables and an
in-memory authoritative saga cache are outside this plan.

## 1. Outcome and boundaries

Implement the complete flow from an accepted start request through structured
offer artifacts, including replay, partial dispatch, retry exhaustion, process
crashes, parallel batches, and independent import failures.

| Stage                                       | Command                          | Success event                |
| ------------------------------------------- | -------------------------------- | ---------------------------- |
| Initialize                                  | `InitializeRetailerFill`         | `RetailerFillInitialized`    |
| Acquire token, if required                  | `AcquireStorefrontToken`         | `StorefrontTokenAcquired`    |
| Retrieve catalog for each requested context | `FetchProductCatalogIdentifiers` | `ProductCatalogIdsRetrieved` |
| Fetch product details                       | `FetchOfferDetails`              | `OfferDetailsFetched`        |
| Extract details                             | `ExtractOfferDetails`            | `OfferDetailsExtracted`      |
| Structuralize details                       | `StructuralizeOfferDetails`      | `OfferDetailsStructuralized` |

There is no GraphQL normalization stage. Import preparation and offers delivery
remain independent subscribers/processes beyond this saga's completion boundary.
Reuse existing retailer handlers and artifact contracts.

Keep PostgreSQL 18 and pinned Mulac 0.5.0. No Mulac release is required. Background
command workers remain disabled; commands and events advance through manual API
batches. Preserve parallel manual batches, including across service processes.

Outside this phase: token refresh or execution-time credential resolution,
automatic cleanup, reopening failed fills, atomic multi-retailer intake,
request-level idempotency, a progress HTTP endpoint, and migration of live v1
fills. Database inspection is sufficient for progress reporting.

## 2. Lessons that change the design

| Observed problem                                                                                                                                                    | V2 requirement                                                                                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Mulac stops event delivery at the first failing subscriber. Import preparation registered before the saga could prevent acceptance of successful structuralization. | Register saga listeners first through one shared production/test registration function. Test the real import subscriber failing through exhaustion. |
| Moving validation into the initialization command regressed synchronous rejection and removed tracking receipts.                                                    | Validate the entire request before enqueueing; return `202` with correlation receipts; move blocking database work off Tokio threads.               |
| Reserved entries survived process death indefinitely. Startup-only recovery also misses reservations that have not expired yet.                                     | Recover at startup and around later batches, protecting active batches against sweeping.                                                            |
| Dedicated recovery connections bypassed the application's connection limit and were reopened after every batch.                                                     | Bound and reuse coordination connections; reuse the batch's connection for its final sweep.                                                         |
| Recovery-enabled and recovery-disabled routes used different execution paths.                                                                                       | One batch runner, explicitly configured during construction, used by production and tests.                                                          |
| One malformed saga aborted observation of every other saga.                                                                                                         | Isolate per-saga data errors; preserve infrastructure errors; keep failures observable without flooding logs.                                       |
| Matching repeatedly scanned products and rebuilt/hashed commands to discover their identity.                                                                        | Persist expected command IDs and perform one direct map lookup, returning a matched-work handle to the transition.                                  |
| Whole-state serialization was repeated for no-op detection and reconciliation counts.                                                                               | Explicit transition results; no serialization for comparison, none on no-op replay, and one payload serialization per changed transaction.          |
| Retained queues were scanned by correlation for every active saga.                                                                                                  | Indexed queue evidence queries scoped to pending work; decode/save each saga at most once per pass.                                                 |
| Passing live fills and CI obscured missing failure tests.                                                                                                           | Prove failure/restart/concurrency contracts before live verification; do not inherit v1's completion claims.                                        |

## 3. Identity and intake contracts

### Identities

- `sagas.id` is an independent RFC UUID v7 and the **sole primary key**. It is
  also the internal namespace for token, catalog, and detail command IDs.
- Enforce `UNIQUE (saga_type, correlation_id)`. Different saga types may share a
  correlation, but may not reuse a saga primary key.
- Route every listener by its saga type and the correlation in event-envelope
  metadata. Never infer correlation from event ID, causation, or a payload.
- Correlation and saga IDs must pass Rust checks and PostgreSQL 18 checks using
  `COALESCE(uuid_extract_version(value), 0) = 7`.
- Causation identifies the immediate parent command/event. It matches an event
  to expected work only after its saga has been located.
- Step payloads, including nested fetched events, contain no `run_id`, `saga_id`,
  or replacement payload `correlation_id`. `KeyedCommand` contains ID and command.
- Preserve the original correlation through every descendant envelope.

### Start request

`POST /api/initial-offer-fills` retains the request shape:

```json
{"retailers":["ankerau","ugreeneu"]}
```

Validate nonempty input, known codes, duplicates, storefront support, and valid
nonempty context configuration for **all** retailers before the first dispatch.
Share configuration validation with the initialization handler. Invalid input
returns `400` and queues nothing. A valid request with no configured bus returns
`503`. Do not perform external storefront HTTP during this validation.

Dispatch and other synchronous database operations run in `spawn_blocking`.
Generate one UUID v7 per retailer, used as initialization command ID and original
correlation. Return `202` in request order:

```json
{"fills":[{"retailer_code":"ankerau","correlation_id":"<uuid>"}]}
```

The receipt identifies a flow whose saga may not exist yet. Do not manufacture
or preallocate its saga ID in the route. A new POST starts a new fill.
Multi-retailer dispatch remains sequential: a queue failure returns `500`, may
follow earlier successful inserts, and is not automatically retried by the API.
Log accepted correlations and the failed command ID; document this partial
acceptance behavior. Update OpenAPI, Bruno examples and existing response tests.

The command handler validates configuration again and emits
`RetailerFillInitialized`; it never accesses saga storage. Configuration may
change between acceptance and command execution. A failure before saga creation
remains an initialization queue failure, not an invented terminal saga.

## 4. Payload storage, lookup and transition cost

### Saga record and schema

Retain `id`, `saga_type`, `correlation_id`, `retailer_code`,
`status IN ('in_progress', 'succeeded', 'failed')`, and an object-valued `payload`.
Use `sagas (id)` in the Diesel schema. Start with an explicit `schema_version: 1`
for this new payload format; unversioned v1 payloads are not silently accepted.
The format version is not a second persistence identity. Keep all configuration, progress, lookup data and counters
inside the payload; there is no separate work/progress table. Index active saga
selection by type/status/ID.

The payload contains:

| Component                  | Contract                                                                                                                                                               |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Version and initialization | Explicit format version; first accepted initialization event and its causation ID.                                                                                     |
| Configuration              | Immutable ordered requested contexts, storefront URL, token policy, and first accepted acquired token when applicable.                                                 |
| Token progress             | Pending/succeeded/failed outcome, canonical result event and diagnostic reason. A policy requiring no token bypasses this item.                                        |
| Catalog progress           | One entry per requested context, its resolved context, outcome, canonical result event, and unique product entries.                                                    |
| Product progress           | One entry per requested-context/product ID; canonical event/object references for fetch, extraction and structuralization, or a failure/skip with reason and evidence. |
| Expected-command lookup    | `command_id -> WorkTarget`, where the target identifies token, catalog, or the exact product and step. IDs are generated once when work becomes expected.              |
| Pending-command set        | IDs of only outstanding work; remove on a first accepted terminal outcome. Retain the expected-command lookup entry for replay and conflict validation.                |
| Counters                   | Pending/failed/skipped work, expected/succeeded catalogs, and expected/structuralized product-context pairs.                                                           |

Keep entries compact. Do not duplicate full commands or nested event payloads.
Canonical fetched/extracted events supply immutable inputs for derived commands.
Resolve catalog context from its saved catalog entry; do not decode a large
catalog event for each product. Keep requested context identity distinct from
resolved context. Catalog acceptance stores the complete deduplicated product set
and expected fetch IDs atomically; a partial expected catalog is not valid state.

Use a hash map for expected-command lookup, a direct context ordinal for catalogs,
and a map keyed by product ID within each catalog. `WorkTarget` is a typed enum
containing the step and the exact context/product location. This is stored lookup
data, not a separately authoritative progress model. Update it and the pending
set in the same transaction as progress. Assert their consistency against progress
in tests and explicit diagnostics; do not rescan/rebuild them on every event.
An existing command ID mapped to a different target is an invariant error, not
a map overwrite. Context ordinals remain stable within the saved snapshot.

### Work creation and counters

Create expected work only when its inputs exist. Initialization creates either
one token item or all requested catalog items. Token acceptance creates catalog
work; catalog acceptance creates fetch work; fetch acceptance creates extraction
work; extraction acceptance creates structuralization work. Every transition
atomically closes its current item and records all successors before evaluating
status. Store each successor's command ID in the lookup at that point.

A failed or evidenced skipped item closes that branch. Do not create imaginary
downstream work or leave it pending. A failed token blocks all requested catalog
coverage; a failed catalog leaves its product population unknown. Both prevent
success without inventing per-product failures. Retain the blocking reason on
the failed token/catalog/product entry.

Apply counter deltas only for the first accepted state change. Closing one pending
item and adding N successors changes pending work by `N - 1`. Duplicate results
change neither counters nor expected work. Outcome evaluation uses the counters,
not repeated scans of every product. Counter/lookup corruption is an invariant
error requiring repair, not permission to guess completion. Enforce cheap local
checks (nonnegative counters, valid target and expected pending membership) on
transitions; complete consistency scans belong to tests/explicit diagnostics.

### Repository and transition interface

Use one transaction connection. Lock the saga row with `FOR UPDATE`, decode its
payload once, perform the transition, and save at most once. Successful event
processing and failure reconciliation use this same transaction boundary.

For a step event:

1. Validate required envelope metadata and locate saga by type/correlation.
2. Look up its causation command ID in the persisted expected-command map.
   Missing saga/work means ignore; do not create a saga or reject another saga's
   retailer. Do not rebuild command IDs while looking up an event.
3. Validate the matched target's event type, retailer, product, context, and source
   artifact references. A matching command with conflicting data is an error.
4. Pass the typed matched-work handle into the transition. Do not perform the
   same lookup again in `apply`, scan other step kinds, or search products again.
5. Accept the first valid result, record canonical references and successors,
   update pending IDs/counters/status, and commit before returning commands.

A direct lookup identifies the expected event type too, so it replaces both the
current `expected_event` scan and the subsequent `locate_*` scan. An E::NAME-first
scan is only an interim optimization, not the final lookup implementation.

Initialization uses create-or-load under the uniqueness constraint and always
checks retailer ownership, including replays. Only the winning first transition
snapshots configuration; replays do not reread inventory.

Return an explicit transition result with `changed` and commands to dispatch.
Centralize state-changing operations so they record their change flag/counter
deltas; do not ask unrelated callers to guess whether arbitrary mutable state
changed. Immutable replay paths report unchanged. An unchanged transition may
still return commands. Serialize the payload only when changed, once at the write
boundary; no `to_value(before/after)` comparisons or whole-payload clones.
Reconciliation reports that same change result rather than comparing state again.
Identity, version and terminal-status guards remain enforced by the repository.

No network calls or object-store reads occur while holding the saga lock.
Canonical event reads use the same database connection and stable event IDs.

### Accepted scaling limit and performance contract

An ordinary product event still reads/decodes the entire payload. A changed event
serializes and writes the entire payload; PostgreSQL JSONB path updates do not
avoid that cost. For P product-context pairs and O(P) result events, cumulative
payload read/write work remains approximately O(P²), with additional linear
payload space for the command lookup. The row lock still serializes transitions
within one saga. This is an explicit accepted limitation of v2.

Within that constraint, require one direct expected-command lookup, direct target
access, no unrelated command construction/hashing, no full-product status scan,
no payload serialization for comparisons, no write on no-op replay, and one
payload serialization/write per changed transaction. Decoding the stored lookup
map is part of the unavoidable whole-payload load; do not describe it as a
constant-time end-to-end transition or an SQL index.

Catalog acceptance/replay is necessarily O(P) for its product set and returned
commands. Reconciliation decodes each visited saga once, iterates its pending
command set, and writes it at most once if changed. These operations must not
load all retained queue history or perform a new full-product scan per pending ID.

Measure payload size, deserialization/serialization, lock time and write volume
at realistic and larger sizes. Report the remaining growth honestly. If it is
unacceptable for the intended fill sizes, stop and propose a new storage ADR;
do not introduce separate progress tables or a cache silently during this phase.

## 5. Stable commands, canonical results and replay

Preserve the existing builders and identities:

- Token: internal saga ID plus command name.
- Catalog: internal saga ID plus command name, requested country/language, and
  historical trailing null cursor. Use the free `initial()` and `keyed()`
  functions in `steps/graphql/product_catalog_ids/reacts_to.rs`.
- Details: internal saga ID plus command name, historical retailer encoding,
  requested country/language, and product ID. Preserve existing serialization;
  do not change it accidentally while changing payload retailer serialization.
- Extraction/structuralization: source object ID plus command name.
- `ApplicationCommand::encode` preserves the keyed ID exactly.

Store generated command IDs in the payload lookup once. Do not regenerate every
product's command to determine which item an incoming event addresses.

For each logical step, the first accepted result event is canonical.
Retries may publish new event IDs and create new immutable raw objects; these do
not replace canonical references, expand the accepted catalog, or fork successors.
Validate conflicting identity fields on duplicates, then use canonical inputs.

Replay returns the original successor set, IDs, and payloads even after later
transitions or configuration changes. Build successors from the matched target
and its saved canonical result; catalog replay enumerates its saved product set
and initialization returns its original roots. Do not search the complete command
lookup to find successors. A zero state-change count must not suppress dispatch. The reaction adapter uses the current triggering event for
envelope causation and preserves correlation; identical command payloads do not
mean newly attempted envelopes must have the original event's causation ID.

Persist progress and successor expectations before dispatch. On transaction
failure dispatch nothing. On dispatch failure return an event-delivery error;
replay returns the full command set and Mulac's retained command IDs suppress
already inserted entries. Do not introduce a dispatch outbox or claim atomicity
between the saga transaction and the command gateway.

Replay lifetime is **indefinite**. Retain command deduplication rows, relevant
trigger/result events, saga payloads, configuration snapshots, and referenced
artifacts. No purge is introduced. Any future cleanup needs a new decision with
an owned replay horizon or durable tombstones and retained reconstruction inputs.
Payload/state versions and stable builders must remain readable for retained
fills; changing their serialization requires a compatibility decision.

Persist the acquired storefront token and replay it unchanged. Deduplication
does not solve stale tokens. Expiry detection, refresh, and execution-time token
resolution remain outside this phase; delayed/newly queued commands may fail
under ordinary retry rules.

## 6. Registration and one batch execution path

Create one registration function used by startup, integration tests, and live
tests. Register all saga subscribers **before** GraphQL step/import subscribers.
Keep one owner of successor dispatch: the saga. Step registration exposes
handlers; remove the old automatic reactions that duplicate saga dispatch.

In Mulac 0.5.0 an event reaches subscribers in registration order and delivery
stops at the first error. Registration order is therefore a correctness
dependency, not formatting. Import failure after saga acceptance cannot undo
structuralization or fail the product. Add a comment and a regression test at
the shared registration boundary.

Construct the bus with its batch runner/recovery coordinator as required private
dependencies. Do not attach `Option<Recovery>` by assigning a field after
`start()`. Tests may inject controlled ports, but must use the same runner.

Both `/commands/run` and `/events/run` call their respective methods on that
runner. Remove the recovery-dependent route branches, unused alternative
adapters, and separate route-level reconciliation hook. Preserve Mulac request
validation, consumers, retry policy, and response formatting. Command/event item
failures remain represented in successful batch responses as today.

## 7. Bounded execution and crash recovery

### Resource ownership

Default to four concurrent manual batches per service process, shared across
command and event routes. `MANUAL_BATCH_CONCURRENCY` is a positive integer,
default 4; invalid values fail startup. Acquire its semaphore permit asynchronously
before spawning blocking work. Waiting requests consume no database connection
and no blocking execution thread. Move the owned permit into the blocking task.

Use a separate bounded coordination connection pool of the same size. Keep the
ordinary application pool bounded at 10 connections by default. The resulting
service budget is at most 10 application plus 4 coordination sessions per process,
excluding external tools. Deployment must budget the sum across replicas; the
semaphore is not a cluster-wide connection limit. No unbounded connection opens
are permitted on any startup, batch, error, or post-batch path.

Reuse a coordination connection across pre-sweep, active batch protection, and
post-sweep; then return it to the coordination pool. Do not call a factory that
opens a second connection when finishing a batch. Ordinary handler connections
must not be held idle as batch guards.

Startup uses the same admission limit and coordination pool; it must not open
an extra uncounted recovery connection. Initialize pool sessions with the same
queue schema/search path as the application.

The pool/guard must track owned session locks. On normal return or unwind, release
all locks it owns before recycling. If cleanup fails or session state is unknown,
discard the connection. Never recycle a locked connection, and never panic in
cleanup. Test this behavior rather than relying on dropping a pooled wrapper,
which alone does not close its database session.

### Coordination protocol

Use the same database-scoped advisory-lock key for command and event batches,
derived from the queue schema/table identity. Independent test schemas must not
share a key. A batch holds a **shared session lock** for the entire actual
consumer execution. Sweeping requires the **exclusive session lock**.

1. Borrow one coordination connection under the admission permit.
2. Try the exclusive lock. If available, sweep both queues, then release it.
   If another batch holds a shared lock, defer sweeping.
3. Acquire the shared lock before reserving any work; execute the batch.
4. Release the shared lock. Try the exclusive lock on that same connection and
   sweep again if available. Never upgrade while retaining the shared lock.
5. Attempt reconciliation even if the final sweep fails. Collect both outcomes;
   report/log infrastructure failures without hiding successful queue execution.
6. Clean up/recycle the connection and release the permit after blocking work and
   observation finish, even if the HTTP client disconnected.

The gap between releasing exclusive and acquiring shared is safe because no
work has been reserved yet. Multiple batches may hold shared locks concurrently.
All consumers sharing these queues must follow this protocol; a mixed rollout
with uncoordinated workers is unsupported.

Use Mulac's command and event sweeps with a five-minute reservation timeout and
the same six-attempt limits as consumers. Use database time through Mulac's
implementation. A timeout alone cannot establish abandonment: Mulac reserves a
whole batch before sequential processing, so live reservations may be old.

Startup runs recovery before reconciliation and before accepting requests, using
the same bounded coordinator. Startup may defer its sweep if another service
process is active. Recent abandoned reservations remain pending until a later
manual batch after the timeout. With continuous overlapping batches, sweeping
may defer until an idle opportunity; operators must allow batches to drain and
run an empty batch to force that opportunity. No timer/worker is added here.

These locks require session affinity. The coordination pool must connect directly
to PostgreSQL or through session-mode pooling. Transaction-mode PgBouncer is not
a supported coordination endpoint. An optional `COORDINATION_DATABASE_URL` may
provide the direct endpoint; otherwise use `DATABASE_URL`. Both endpoints must
resolve to the same database and queue schema. Do not promise automatic detection
of an incorrectly configured proxy or use transaction-scoped locks as a silent
substitute for a batch-long session lock.

## 8. Failure reconciliation and observability

Reconcile after every manual batch, including empty batches and batches reporting
item failures, and at startup after recovery. Serialize overlapping reconciliation
passes per process to avoid duplicate scans; database row locks preserve
cross-process correctness. A later pass must still run after the requesting
batch's outcomes are committed.

Visit active saga IDs in stable order. For each saga, take its row lock, decode
once, and examine the pending-command set through direct targets. Query queue
evidence in chunks of at most 256 pending IDs, updating the decoded state through
the same transition functions. Evaluate counters/status and write once at the end
only if changed. Do not release/reacquire and decode the whole payload for every
chunk. A single saga pass still holds its lock for O(payload + pending work);
chunking bounds queue-query size, not saga memory or lock duration.

Use one database statement/snapshot per chunk to gather the relevant commands,
result events, and trigger-delivery evidence. Commands use their primary IDs;
result/duplicate-trigger lookups use correlation, causation, and event type.
Compare JSON metadata strings against bound UUID text rather than casting every
unrelated metadata value to UUID.

Add expression indexes matching those predicates to command/event tables. Include
correlation access and an event index on correlation/causation/event type. Explain
the actual queries against representative retained history; having an index by
name is not evidence that the query uses it. Sweeping still follows Mulac's queue
state rules and existing reservation indexes.

| Evidence for pending work                                                                       | Required behavior                                                                                      |
| ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Already accepted canonical result                                                               | Preserve it; observe only still-pending successors.                                                    |
| Queued, reserved, or retryable command                                                          | Remain pending; recovery handles abandoned reservations.                                               |
| Published matching result still deliverable                                                     | Wait for delivery, even if the command is exhausted.                                                   |
| `Failed` with attempts at the limit, or `Dead`, with no deliverable result                      | Record failure; do not wait for `Dead` to appear.                                                      |
| Completed command without any expected result                                                   | Record missing-result failure.                                                                         |
| Result delivery exhausted without an accepted outcome                                           | Record unapplied-result failure.                                                                       |
| Expected successor absent from queue but its triggering event or equivalent duplicate can retry | Wait for replayed dispatch.                                                                            |
| Expected successor absent and all relevant trigger delivery is terminal                         | Fail only the undispatched successor; retain accepted parent progress and already dispatched siblings. |
| Parent was accepted, independent subscriber exhausted                                           | Do not fail the parent/product; separately evaluate missing successors if any.                         |
| Explicit disappearance/unpublication with evidence                                              | Record a skip; arbitrary HTTP/storage errors are failures, never inferred skips.                       |

Use the same transition/counter rules as successful event processing. Missing
canonical inputs indicate retention/invariant damage, not permission to rebuild
from live configuration or fabricate a success.

Return an internal report with scanned/changed/failed saga counts and swept queue
counts. Log saga ID, type, correlation, work/queue IDs and bounded reasons, never
tokens or whole command payloads. Count each saga once per pass, regardless of
queue-query chunks. Queue HTTP response shapes remain unchanged apart from
intake's `202`.

Malformed/unsupported saga state and other per-saga data errors roll back that
saga's transition and do not stop others or prevent startup. Leave its business
status unchanged; unreadable state is not evidence of business failure. Retry on
later passes so a compatible deployment or repair can recover it.

Log the first `(saga ID, state version, error fingerprint)` immediately, then
at most once per five minutes for unchanged errors; keep per-pass failure totals.
Use a bounded cache (1,024 entries per process), log changed errors immediately,
and clear an entry on successful reconciliation. Restart/eviction may produce
another diagnostic. Document operator repair and retry; no automatic quarantine
status or schema-default guessing is introduced.

Connection acquisition failures, database outages and invalid infrastructure
queries remain visible failures: startup fails, or a batch returns a sanitized
`500`. Attempt all applicable post-batch cleanup/observation before returning;
log both sweep and reconciliation failures when both occur.

## 9. Terminal outcome policy

Keep exactly three saga statuses: `in_progress`, `succeeded`, `failed`.

- A fill remains in progress while required planned work is pending.
- A terminal fill succeeds only when all requested catalogs succeeded and every
  discovered product/context reached structuralization, with no failures/skips.
- Any failed or skipped branch makes a terminal fill failed. Token-policy bypass
  is not a skipped product; tokenless fills can succeed.
- Empty catalogs are valid successful outcomes. Missing/empty requested context
  configuration is invalid intake, not an empty successful fill.
- Upstream failure closes its uncreated downstream branch; other already
  planned branches must finish or fail before overall finalization.
- Counter/coverage inconsistencies are invariant errors, not a reason to guess
  a terminal status. Verify counters and pending IDs against payload progress in tests/diagnostics.
- Terminal status never reopens. Previously accepted outcomes still replay their
  canonical successors; late results for terminal failed/skipped work do not
  replace those outcomes. Start a new fill to retry business work.

Queue emptiness, a zero-command transition, and “all products accounted for”
alone are not success criteria.

## 10. Reset, migrations and documentation

Before any implementation reset, capture the exact branch/base, working-tree
changes, useful tests, and schema versions. The user chooses the reset boundary;
do not infer authorization to discard work or reset databases from this plan.
Preserve reusable handlers, builders, fixtures and the regression cases as
specifications. Do not blindly copy the current bus/recovery architecture.

Implement against a fresh isolated PostgreSQL 18 schema with empty queues.
Recheck which migrations are unshipped relative to the chosen base. Only unshipped
history may be consolidated into the final schema/index migration. Otherwise add
forward migrations. Previously applied development schemas require an explicit
recreation/migration decision; changing migration source does not update them.
Do not add a nominal rollback chain for a schema that never shipped.

Add payload-version/constraint checks, saga-selection and queue indexes
from the first storage slice. Down migrations drop only objects introduced by
that migration. Do not migrate or silently reinterpret active v1 fills. Stop/drain
old consumers before switching an environment to v2. If an environment must keep
replaying existing v1 fills, the fresh-baseline assumption does not apply: prepare
a separate compatibility/migration plan before deployment. Draining queues alone
does not waive the retention/replay contract.

Record persisted payload lookup, explicit change tracking, accepted scaling costs
and bounded batch/recovery ownership in a new ADR superseding the relevant parts of [ADR 93](../adr/0093-saga-storage-and-identity.md)
and [ADR 94](../adr/0094-saga-execution-replay-and-recovery.md). This plan proposes
those changes; it does not itself edit accepted ADRs. Avoid a succession of
accepted ADRs describing intermediate implementations.

Update the service README, saga walkthrough, API examples, root/service TODOs,
and development metadata consumers. Listings and dump grouping use envelope
correlation and preserve separate context paths/artifact identities. Keep v1's
historical verification labeled as historical; it is not evidence for v2.

## 11. Delivery order and acceptance gates

Each slice has its own TODO marker and tests. Finish planning at `[>]`; move to
`[~]` only when implementation starts and `[x]` only after its gate passes.

1. **Baseline and contracts.** Confirm reset/migration boundary; lock identity,
   intake, token, terminal-outcome and storage contracts; preserve known failures
   as regression cases. No unrelated handler refactor.
2. **Payload storage and lookup.** Implement saga schema, constraints, transaction
   API, persisted expected-command lookup, change tracking, counters and version
   handling. Prove no-op writes and concurrent finalization before adding the full pipeline.
3. **One bounded runner.** Implement explicit construction, shared registration,
   admission control, pooled guard ownership and recovery. Exercise contention,
   cancellation, panic and cleanup before using this runner in production.
4. **Intake and initialization.** Restore full validation and receipts, share
   handler validation, and create/recover the saga from initialization metadata.
5. **Transitions and replay.** Add token and tokenless paths, atomic catalog
   fanout, product steps, canonical results and replay after partial dispatch.
6. **Failure observation.** Add indexed queue evidence queries, exhausted/missing
   work handling, per-saga isolation, throttled diagnostics and reporting.
7. **Scale and compatibility checks.** Verify query plans, resource bounds,
   operation counts, payload contracts and metadata consumers. Resolve failures
   before declaring the implementation ready for live fills.
8. **End-to-end evidence and documentation.** Run all required checks and both
   live paths, inspect every artifact, update decisions/tracking to the final
   design, and record measured results with commands/environment used.

Implementation entrypoints are the saga model/listener/repository, platform
bus/batch API/recovery, GraphQL step builders/registration, migrations, and
development metadata consumers. Preserve useful current regression cases in
`transition_tests.rs`, `recovery.rs`, and the API pipeline tests, but rebuild their
fixtures against v2 invariants. Fixtures must query sagas by explicit identity,
set queue statuses deliberately, and avoid bypassing the production runner.

### Required verification matrix

| Area                 | Acceptance evidence                                                                                                                                                                                                                                                  |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Schema               | PostgreSQL 18 migration up/down/up; UUID version/variant checks; sole saga PK; same-correlation different types; conflicting owner rejected; unique catalog/product-context entries and expected command targets; no destructive reset of a populated environment.   |
| Intake               | Empty/unknown/duplicate/unsupported/mixed requests; invalid configuration queues nothing; `202` receipts match queued envelope correlations; queue outage/partial insertion; no blocking inserts on an async runtime thread.                                         |
| Matching             | Distinct saga and correlation IDs; absent/invalid metadata; missing saga; shared correlation across types; unmatched wrong retailer ignored; matched wrong retailer/type/context/product/source rejected; one lookup reused by application.                          |
| Canonical replay     | Duplicate events and distinct event IDs from command retries; changed configuration and duplicate result content; stable command IDs/payloads at every stage; accepted product set unchanged; no-op replay produces commands without row updates.                    |
| Dispatch             | Crash after commit before dispatch and failure halfway through catalog fanout; retry inserts only missing commands; retries exhausted with missing successors; pending duplicate trigger can still recover them.                                                     |
| Progress             | Empty catalog; duplicated product IDs; same product in distinct requested contexts; explicit skip; failed token/catalog/product; concurrent final results; counter invariants; terminal fills cannot reopen.                                                         |
| Subscriber ordering  | Production registration, actual import subscriber failing from the first structuralization delivery through six attempts; saga still succeeds and failure remains visible in the event queue.                                                                        |
| Failure evidence     | Retryable versus exhausted commands/events; publication before command failure; pending result wins; accepted result wins; missing result; blocked downstream work; malformed saga beside healthy progressing sagas; infrastructure errors remain visible.           |
| Recovery             | Expired and recent abandoned command/event reservations; startup and later empty batch; two processes with overlapping long batches; no sweep of live work; eventual recovery after they drain; shared-to-exclusive transition does not deadlock.                    |
| Bounds and cleanup   | Fifty overlapping requests with limit four: at most four batch tasks/guard sessions; waiters use no connection; no connection creation on each successful reuse; no second post-batch connection; pool size and replica budget verified.                             |
| Cancellation         | Cancel while waiting, during blocking consumption and after consumption; permits/locks remain owned until work ends; panic/unlock failure discards or cleans a session; next checkout cannot inherit locks; reconciliation still attempted after post-sweep failure. |
| Queue indexes        | Representative retained unrelated/terminal queue history; `EXPLAIN (ANALYZE, BUFFERS)` for actual evidence predicates; indexed queue evidence lookup without full-history loading; reversible index migration.                                                       |
| Retention/versioning | Demonstrate why deleting command rows permits re-execution and deleting canonical events breaks reconstruction; prohibit both; incompatible state fails visibly per saga, compatible version replay remains deterministic.                                           |
| Diagnostics          | Corrupt saga reports do not flood logs, totals remain visible, repair resumes observation, cache stays bounded, tokens/payloads never leak.                                                                                                                          |
| Production parity    | Startup, route tests, failure tests and live harness share the registration and runner construction; no branch chooses a legacy execution path when recovery is absent.                                                                                              |

Use synthetic fills of 100, 1,000 and 10,000 products in one context, plus a
multi-context case. Measure ordinary transition statement count, rows/bytes
read and written, lock hold time, catalog fanout time and reconciliation work.
The structural gate is a single direct match, no repeated scans or comparison
serialization, no no-op writes, and bounded connection use. Explicitly measure
whole-payload decoding, one serialization/write for changed transitions and the
remaining quadratic cumulative cost. Timings are reported with hardware and
environment rather than asserted against arbitrary millisecond limits.
Measure catalog fanout separately. A warm in-memory cache is not required to pass.

Run `make fmt`, then `make ci` in affected workspaces, the PostgreSQL 18 suite,
and relevant smoke tests. Run `make audit` only if Cargo dependencies/lockfiles
change. Start the service with `PORT=<port> make serve`; check health, startup
recovery, both empty batch routes, intake validation and receipts.

Finally run two full API-driven fills: Anker AU (acquired token) and Ugreen EU
(tokenless), against isolated schemas and real object storage. Drive both queues
through the production runner; include concurrent batches from a second process.
Require `succeeded` and inspect every structured artifact for product identity,
requested/resolved context, variants, currency and standard/discount pricing.
Retain non-secret manifests. Product counts may change; v1's 152/1,234 counts are
historical observations, not expected constants.

Do not mark v2 implemented merely because a happy-path crawl finishes. All
failure, recovery, isolation and resource-bound gates above must pass. Writing
this Markdown plan alone requires no `make` commands under repository guidance.
