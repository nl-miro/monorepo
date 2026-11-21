# Saga implementation v3

Status: `[>]` planned. This is the single active saga plan; it supersedes the
historical [Saga implementation](saga-implementation.md). Implementation and any
reset have not started.

## Why redo the implementation

The first implementation proved the design. It lives on branch
`wip/refactoring-to-sagas-and-steps`, in commits `e0546a73` and `ffb86582`. On
2026-11-20, both live fills started through the API reached `succeeded`:

- Anker AU, with an acquired storefront token: 152 structured artifacts.
- Ugreen EU, without a token: 1,234 artifacts across five contexts.

It also showed where the plan was incomplete. Review found four problems:

- A failing import preparation step could fail products that had finished.
- The start endpoint had lost its validation and its receipts.
- A crash left sagas `in_progress` forever.
- One malformed saga blocked every batch.

Fixing these after the fact meant extra recovery code and a follow-up
migration. It also meant six ADRs that later had to be merged into two.
Everything landed in one 230-file commit.

v3 restarts the saga work from the reorganized module layout. It keeps the
design that worked, settles the gaps up front, and delivers the work in
reviewable commits.

## Decisions

| Topic              | v3 decision                                                                           | Compared with v1                     |
| ------------------ | ------------------------------------------------------------------------------------- | ------------------------------------ |
| Starting point     | Module layout of ADRs 91 and 92; saga work redone                                     | New                                  |
| Progress storage   | One `sagas` row per saga with a JSONB payload; no progress table                      | Same; lookups and no-op writes fixed |
| Routing            | Saga type plus envelope correlation; causation matched before the retailer is checked | Order changed                        |
| Command IDs        | UUID v5 namespaced by the saga ID; keys drop the parts kept only for compatibility    | Simplified                           |
| Dispatch           | The saga owns every next-step command; replay returns the same commands               | Same                                 |
| Subscriber order   | Saga listeners register before independent subscribers, through one function          | New                                  |
| Failure handling   | Queue reconciler, isolated per saga, backed by indexes for pending-work evidence                    | Extended                             |
| Stale reservations | Swept at startup and around manual batches, coordinated by advisory locks             | New                                  |
| Queue driving      | Manual batches only; `COMMAND_WORKERS` removed                                        | Changed                              |
| Skips              | Model kept; nothing produces skips in this phase; strict success                      | Clarified                            |
| Start API          | Every retailer validated before anything is queued; `202` with correlation receipts   | Restored and extended                |
| ADRs               | Written once, after the implementation settles                                        | Changed                              |

## Starting point

v3 starts from a tree that has the module reorganization from `e0546a73` and
none of its saga work:

- [ADR 91](../adr/0091-sitemap-workflow-steps.md) and
  [ADR 92](../adr/0092-business-processes-sagas-and-steps.md), plus the service
  AGENTS.md section that describes business processes, sagas and steps.
- `src/business_process/`, `src/saga/` with the sitemap sagas, and
  `src/steps/{graphql,html,sitemap,record_export_messages}`.
- GraphQL steps that behave as they did in commit `11783165`:
  - Payloads and `KeyedCommand` carry `run_id`.
  - Steps dispatch the next command through automatic reactions
    (`Bus::on_many_keyed`).
  - The v2 fill service (`start_fills`) creates the run ID and dispatches the
    first commands.
- None of these exist yet:
  - `InitializeRetailerFill`
  - `on_commands_envelope`
  - `after_batch`
  - the `sagas` migration
  - the saga repository, listener, reconciler and recovery code
  - ADRs 93 and 94
  - the saga sections of the README and TODO

Preparing that starting tree is Phase 0, not an action performed by editing this
plan. Preserve the selected commits and useful regression tests before resetting;
record the exact extracted baseline and migration state. Treat the old saga tests
and recovery code as reference material to re-create, not files present in the
baseline. Do not reset a populated database without an explicit decision.

Before Phase 1, confirm that `make ci` passes on that tree. Also confirm that
nothing named `saga_repository`, `recovery` or `sagas` remains under the
service's `src/` or `migrations/`. If the extracted tree has already removed
`run_id` or the reactions, skip those Phase 3 and Phase 4 items.

## What we learned

### Mulac behavior the design depends on

1. **Event fan-out stops at the first error.**
   - Mulac 0.5.0 stores one `event_entries` row per event and runs its
     subscribers in registration order.
   - The first error stops delivery and fails the row
     (`mulac-kernel-0.5.0/src/event_subscriber_registry.rs:38`). Mulac keeps
     no delivery state per subscriber.
   - A subscriber that fails before the saga hides the event from the saga. A
     subscriber that fails after it makes the saga receive the event again.
   - So subscriber order is part of the design, and saga listeners must handle
     a repeated event safely.
2. **Nothing sweeps stale reservations.**
   - Mulac ships `StaleCommandSpec` and `StaleEventSpec`, but no code calls
     them.
   - A process killed mid-batch leaves rows `Reserved` forever, and their saga
     never finishes.
   - A sweep marks expired reservations `Failed` and keeps their attempt count.
     Work that keeps crashing the process still runs out of attempts.
3. **Retries end at `Failed`, not `Dead`.**
   - The HTTP runner takes only a `limit`, so Mulac's default of six attempts
     applies. A row is exhausted when it is `Failed` with six or more attempts.
   - Use `ReservableCommandSpec::DEFAULT_MAX_ATTEMPTS` and
     `ReservableEventSpec::DEFAULT_MAX_ATTEMPTS`. Never wait for `Dead`.
4. **Queue inserts deduplicate.**
   - Command inserts use `ON CONFLICT DO NOTHING`.
   - So returning the full command set on every replay recovers a partial
     dispatch without an outbox. This works only while completed command rows
     are kept.
5. **Mulac already propagates correlation.**
   - `HandlerWrapper` copies command correlation into events, falling back to
     the command ID.
   - Envelope-aware subscriptions give listeners that metadata.

### Design lessons

6. **The reconciler's rules were right.** Keep them exactly (see
   [Failure handling](#failure-handling)). The race cases hold under the saga
   row lock.
7. **One bad saga must not stop the others.** A malformed payload stopped the
   startup pass, and every later batch returned `500`.
8. **Queue lookups need indexes.**
   - The reconciler filters both queue tables on `meta->>'correlation_id'`,
     and queue rows are kept indefinitely.
   - Without expression indexes, every batch scanned both tables once per
     running saga, while holding that saga's lock.
9. **Avoid repeated command-ID scans; measure the whole-payload cost.**
   - A 1,000-product payload measured 390,251 JSON bytes. One locked replay
     took about 43 ms locally.
   - Every event recomputed every product's command ID to find its entry,
     and the expected-work check repeated that up to four times. Remove these
     scans without assuming the local measurement establishes a throughput
     limit or proves payload writes cheap at every catalog size.
   - Unchanged transitions also rewrote the row.
10. **Match the work before checking the retailer.** Checking the retailer
    first rejects an event that belongs to another saga type sharing the same
    correlation.
11. **The start endpoint needs a written contract.** During the rewrite it lost
    pre-dispatch validation, returned `204` with no way to find the saga, and
    blocked a Tokio thread.
12. **Recovery has a resource cost.**
    - Each batch opened up to two connections outside the pool, with no upper
      limit.
    - Recovery was attached by setting a field after `Bus::start`.
    - `bus_api` kept a second batch path that only tests used.
13. **Compatibility keys had nothing left to stay compatible with.**
    - The detail key kept `format!("{retailer:?}")`, and the catalog key kept a
      trailing null cursor, so IDs would match those from before the saga.
    - Namespacing by the saga ID already makes every ID new.
14. **Tests must call production builders.** Some tests checked test-only
    copies instead: `fetch_details_for_catalog_products`,
    `start_catalog_for_locales` and a test-only `Bus::on_many_keyed`.
15. **Leftover modules confuse ownership.** `src/v2/application` ended up
    holding only tests, about 2,100 lines. Saga code imported
    `InitialFillContexts` from `crate::v2`.
16. **An unused skip model hides a policy gap.** Nothing ever produced a skip,
    so one product unpublished during a fill makes the whole fill `failed`.
17. **Docs written during implementation go stale.**
    - Six ADRs had to be merged into two.
    - The README said four workers start, while `main.rs` hard-coded zero.
    - Bruno requests still described `204` and `run_id`.
    - TODO entries linked to plan sections that no longer existed.

## Design

Paths under `src/` are relative to `services/retailer_sourcing`.

### Saga record and schema

One migration creates the `sagas` table. A second creates the queue
evidence indexes. The second touches existing tables, so it gets its own
down migration.

```sql
CREATE TABLE sagas (
    id UUID PRIMARY KEY CHECK (COALESCE(uuid_extract_version(id), 0) = 7),
    saga_type TEXT NOT NULL,
    correlation_id UUID NOT NULL
        CHECK (COALESCE(uuid_extract_version(correlation_id), 0) = 7),
    retailer_code TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('in_progress', 'succeeded', 'failed')),
    payload JSONB NOT NULL CHECK (jsonb_typeof(payload) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (saga_type, correlation_id)
);
CREATE INDEX sagas_status_idx ON sagas (status);
```

```sql
CREATE INDEX command_entries_correlation_idx ON command_entries ((meta->>'correlation_id'));
CREATE INDEX event_entries_correlation_idx ON event_entries ((meta->>'correlation_id'));
CREATE INDEX event_entries_result_idx ON event_entries
    ((meta->>'correlation_id'), (meta->>'causation_id'), event_type);
```

- **UUID checks.** `uuid_extract_version()` returns NULL for a UUID without the
  RFC 9562 variant, and a CHECK passes on NULL. `COALESCE` turns that into a
  failure, so one expression checks both version and variant. This needs
  PostgreSQL 18, locally and in CI.
- **Timestamps.** `created_at` and `updated_at` are new. They show how long a
  saga has been stuck without decoding its payload.
- **Status.** Model it as a Rust enum, `SagaStatus { InProgress, Succeeded,
  Failed }`, stored as the text values above. A terminal status never changes.
- **IDs.** The saga ID comes from `Uuid::now_v7()`. Validate both UUIDs in Rust
  as well as in the database.
- **Identity.** It lives only in the record. The payload holds no saga ID,
  correlation or retailer.

### Repository

`SagaRepository` runs every change as one transaction on one connection:

1. Initialization only: insert the record with
   `ON CONFLICT (saga_type, correlation_id) DO NOTHING`.
2. Lock the row by saga type and correlation with `SELECT ... FOR UPDATE`.
3. Run the transition with the record and the same connection.
4. Reject any identity change and any change to a terminal status.
5. Save only if the payload or status changed, and set `updated_at`.

It has two entry points:

- `initialize(correlation, retailer, snapshot, apply)` creates or loads the
  record. It rejects a stored record with a different retailer.
- `transition(correlation, apply)` loads the record without a retailer filter.
  The listener checks the retailer only after it matches the event to expected
  work.

Require `S: Saga + Clone + PartialEq`. Decode the payload once and detect changes
by comparing with one clone taken before the transition, not by serializing the
payload. Return whether the transaction changed state along with its commands;
the reconciler uses that result for its changed count rather than making another
clone/comparison. Serialize once at the write boundary for changed state, and
never serialize just to detect a change. An unchanged replay still returns its
commands and leaves both the row and `updated_at` untouched.

### Payload

Keep the shape the first implementation settled on:

```rust
struct ResultRef { event: Uuid, object: Uuid }
struct Skip { reason: String, evidence: String }
struct Product {
    fetched: Option<ResultRef>,
    extracted: Option<ResultRef>,
    structuralized: Option<ResultRef>,
    failure: Option<String>,
    skipped: Option<Skip>,
}
struct Catalog {
    context: RequestedContext,
    resolved: Option<RequestedContext>,
    event: Option<Uuid>,
    products: BTreeMap<String, Product>,
    failure: Option<String>,
}
struct ShopifyGraphQlInitialFillSaga {
    schema_version: u32,
    catalogs: Vec<Catalog>,
    acquire_token: bool,
    storefront_url: String,
    initialization: Option<Uuid>,
    token_event: Option<Uuid>,
    storefront_token: Option<StorefrontToken>,
    token_failure: Option<String>,
}
```

- **Format version.** New payloads have `schema_version: 1`. Validate the version
  before decoding its state. Missing or unsupported versions are per-saga data
  errors, not permission to guess defaults. Retained payloads must remain
  replayable: future format changes require an explicit reader/migration policy.
  This fresh baseline does not silently reinterpret old, unversioned saga rows.
- **Snapshot.** Taken once at initialization: the requested contexts, whether
  to acquire a token, and the storefront URL. Replays never read the retailer
  configuration again.
- **Keys.** Catalogs keep the order of the requested contexts. Products are
  keyed by product ID inside their catalog, so the same product in two contexts
  is two entries.
- **Pending work.** A missing result means that step is still pending. The
  latest accepted result shows which step comes next.
- **Canonical results.** The first accepted result for each step is canonical.
  A retried command can publish a new event and object; later results reuse the
  canonical one, so the chain of work never forks.
- **Catalog results.** A catalog result records its full set of unique products
  in the same transition.
- **Skips.** `Skip` stays in the model, but nothing produces it in this phase
  (see [Success criteria](#success-criteria)).

All progress stays in this JSON payload. Do not add a stored command-ID map,
pending-ID set, or completion counters; derive matching, pending work and status
from the authoritative progress entries. No separate progress table is introduced.

Every event still decodes and clones the whole payload; every changed transition
serializes and rewrites it. No-op replay avoids the write. A status scan and the
`PartialEq` comparison remain proportional to payload size. Across O(P) events
for P products, cumulative payload work remains approximately O(P²). Accept that
limit for this phase and measure it at increasing sizes. The lookup improvement
does not make the entire transition constant-time. See
[Out of scope](#out-of-scope) for when to revisit storage.

### Command identities

| Command                          | Namespace                     | Key                                                                           |
| -------------------------------- | ----------------------------- | ----------------------------------------------------------------------------- |
| `AcquireStorefrontToken`         | saga ID                       | command name                                                                  |
| `FetchProductCatalogIdentifiers` | saga ID                       | command name, requested country, requested language                           |
| `FetchOfferDetails`              | saga ID                       | command name, `retailer.as_str()`, requested country and language, product ID |
| `ExtractOfferDetails`            | canonical fetched object ID   | command name                                                                  |
| `StructuralizeOfferDetails`      | canonical extracted object ID | command name                                                                  |

- IDs come from `stable_command_id` in `src/platform/bus.rs`.
- Using the requested context rather than the resolved one keeps IDs distinct
  when two contexts resolve to the same values.
- Step payloads carry no `run_id`, `saga_id` or correlation.
- `ApplicationCommand::encode` keeps the ID the builder produced.

### Matching an event to its work

Use the event payload only to narrow down candidates. Then confirm each
candidate by recomputing its command ID and comparing it with the envelope's
causation ID.

| Event          | Candidates                                                                               |
| -------------- | ---------------------------------------------------------------------------------------- |
| Token          | The token command                                                                        |
| Catalog        | Each catalog command, one per requested context                                          |
| Fetched        | The event's GraphQL product ID, in each catalog that contains it                         |
| Extracted      | The product ID from the nested fetched event, where the canonical fetched object matches |
| Structuralized | The same product ID, where the canonical extracted object matches                        |

- Candidate enumeration grows with the number of catalogs. Product access is a
  `BTreeMap` lookup in each relevant catalog; never scan every product or build
  a command for every product. Compute candidate IDs from persisted saga
  identity/retailer/context and canonical source objects, not untrusted event
  retailer values. The payload only narrows candidates; causation confirms them.
- Return the matched catalog/product location to the transition. Do not repeat
  matching in `apply` or persist a command-ID index.
- Each listener matches only its own step. A step command produces exactly one
  event type, so an event can't match another step's work, and no scan is
  needed to check.
- If nothing matches, return no commands and change nothing.
- If the work matches but the event names a different retailer, reject the
  event. Validate the matched context/product and canonical source references
  before accepting its result. Keep matching within the listener's own step.

### Listeners and transitions

Every saga listener requires envelope metadata with a UUID v7 correlation. Step
events also require a causation ID. Never fall back to payload fields, event
IDs or the saga ID.

- **Initialization.** `RetailerFillInitialized` creates or loads the saga from
  a snapshot. It returns the token command, or the catalog commands for a
  retailer that needs no token. A repeated event with the same correlation
  reuses the saga and returns the same commands.
- **Step events.** Token, catalog, fetched, extracted and structuralized events
  each match their work and record the first accepted result. The listener then
  recomputes the status, commits, and only then returns commands.
- **Replays.** A replay returns the same commands even after the saga has moved
  on. Nested commands are rebuilt from canonical events loaded from
  `event_entries` (`load_result`).
- **Terminal sagas.** A replay never reopens a terminal saga. To retry, start a
  new fill.

### Registration

`register_graphql_pipeline(bus, platform)` in `src/main.rs` registers, in this
order:

1. The saga listeners.
2. The GraphQL step handlers.
3. Import preparation, which stays an independent subscriber.

`main` and every pipeline test call this function, and nothing else registers
these pieces. Steps register handlers only. The saga is the only code that
dispatches next-step commands.

### Starting a fill

`POST /api/initial-offer-fills` takes a body like
`{"retailers":["ankerau","ugreeneu"]}`.

| Status | When                                                                                                   |
| ------ | ------------------------------------------------------------------------------------------------------ |
| `202`  | Every retailer's initialization command was queued; the body lists one receipt per retailer            |
| `400`  | Empty list, unknown or duplicate code, or invalid storefront configuration or contexts; nothing queued |
| `500`  | Queueing failed; earlier retailers in the same request may already be queued                           |
| `503`  | No command bus                                                                                         |

The `202` body looks like this:

```json
{ "fills": [{ "retailer_code": "ankerau", "correlation_id": "<uuid>" }] }
```

- **Validation.** It runs for every retailer before anything is queued, through
  `validate_retailer`. The `InitializeRetailerFill` handler uses the same
  function. Validation reads local configuration only; no storefront HTTP is
  performed during intake. Keep handler validation too because configuration
  can change between acceptance and command execution.
- **Correlation ID.** The receipt's correlation ID is the
  `InitializeRetailerFill` command ID.
- **Execution.** Dispatch runs in `spawn_blocking`. The saga is still created
  later, when the initialization event is processed.
- **No atomicity.** A request for several retailers is not atomic. On a partial
  dispatch failure, log the already accepted retailer/correlation receipts and
  the failed command ID. Do not log tokens or full command payloads, automatically
  retry the whole request, or claim the earlier inserts were rolled back.
- **Old path.** The v2 `start_fills` path is removed.

### Failure handling

The reconciler runs at startup after recovery and after every manual batch,
including empty batches and batches with item failures. Allow at most one pass
at a time per service process; concurrent callers queue for that pass rather
than silently skipping observation. Database row locks protect passes across
processes. A post-batch sweep failure must not prevent an attempt to reconcile.

Visit active saga IDs in stable order. Under each saga's row lock, decode its
payload once and walk progress once to derive pending command IDs and canonical
trigger references. This list is temporary, not persisted in the payload. For
products, the latest accepted result determines the one next step; completed,
failed or skipped branches need no queue evidence.

Query evidence in chunks of at most 256 pending command IDs. Each chunk uses one
statement/snapshot to obtain the relevant command rows, matching result events,
and trigger/duplicate-trigger delivery state required by the rules below.
Command IDs use the queue primary key; event lookup uses correlation, causation
and event type. Fetch status/attempts/reasons and identity metadata, not all event
payloads or every historical queue row for the correlation. Load canonical event
payloads by ID only when reconstruction requires them.

Compare metadata strings with bound UUID text rather than casting unrelated
JSON values to UUID. Use the correlation/causation/type event index above and
verify the actual query plans with representative retained queue history. The
index migration's down path removes all indexes it introduces.

Keep the saga transaction/decoded payload across chunks. Apply all observations,
recompute status once, and save at most once if changed. Chunking limits query
size; it does not eliminate the whole-payload memory cost or lock duration.
Report the repository's changed result rather than serializing/comparing again.

For each pending piece of work, apply the first matching rule:

| #   | Queue state                                                                | Outcome                                                   |
| --- | -------------------------------------------------------------------------- | --------------------------------------------------------- |
| 1   | A result event is neither exhausted nor completed                          | Wait                                                      |
| 2   | The command exists and is neither exhausted nor completed                  | Wait                                                      |
| 3   | The command finished and published a result, but no outcome was accepted   | Fail: "result delivery ended without an accepted outcome" |
| 4   | The command is exhausted                                                   | Fail: "command retries exhausted"                         |
| 5   | The command completed without a result                                     | Fail: "command completed without a result"                |
| 6   | No command row, and another result of the same upstream command is pending | Wait; replaying that result can still dispatch the work   |
| 7   | No command row, and the event that should have dispatched it is finished   | Fail: "successor … was not dispatched"                    |

- **Precedence.** An accepted outcome always wins over a queue failure.
- **Blocked work.** Downstream work blocked by a failure ends with its product
  or catalog.
- **Independent subscribers.** Their failures can't undo saga work that is
  already committed.
- **Bad payloads.** A saga whose payload can't be decoded or validated is
  counted and left unchanged while other sagas proceed. Unreadable state is not
  a business failure. Retry on later passes so a repaired state or compatible
  deployment can recover it; no automatic terminal/quarantine status is added.
- **Database errors.** They still fail the batch with a retryable `500`, or
  stop startup.
- **Logging.** Each pass logs distinct saga counts (scanned, changed, failed)
  and recovery counts, even when individual diagnostics are throttled. Log the
  first per-saga error immediately, then suppress the same error for five minutes;
  changed errors log immediately and successful reconciliation clears the entry.
  A small process-local map of saga ID to last error fingerprint/time suffices;
  cap it at 1,024 entries and evict the oldest when full. No new logging service
  or persistent error state is needed. Include ID, correlation and type without
  token/payload data. Document repair followed by an empty batch retry.

### Stale reservation recovery

- **Admission.** `MANUAL_BATCH_CONCURRENCY` is a positive integer, default `4`;
  invalid values fail startup. One semaphore is shared by command and event
  batches per process. Acquire an owned permit asynchronously **before**
  `spawn_blocking`; waiting requests hold neither a blocking thread nor a database
  connection. Move the permit into the blocking task and retain it through batch
  execution, recovery, reconciliation and cleanup, even after client cancellation.
- **Batch lock.** Each admitted batch opens one dedicated connection and holds
  a shared session advisory lock for actual consumer execution. Take the lock
  before reserving work, inside the blocking task. Never use an idle application
  pool connection as the batch guard.
- **Sweep lock.** Try the exclusive lock with `pg_try_advisory_lock`. If another
  batch holds a shared lock, defer the sweep. Sweep command and event reservations
  older than five minutes using Mulac's database clock and consumer attempt limits.
  Age alone is insufficient: Mulac reserves a whole batch before processing its
  items sequentially, so live reservations can be old.
- **Lock key.** A constant plus the OID of `command_entries`, scoped to the same
  database/schema for commands and events. Isolated test schemas must not block
  each other.
- **When to sweep.** At startup before reconciliation; before a batch reserves
  work; and after it finishes. Release the shared lock before trying the exclusive
  lock on the **same guard connection**. Never open a second connection or try to
  upgrade while holding shared. No work has been reserved during the gap between
  the pre-sweep and acquiring shared, so that gap is safe.
- **Cleanup.** Close the dedicated connection when the blocking task ends, including
  unwind/error paths; closing releases session locks. Do not pool guard sessions
  in this phase. Reusing one connection within a batch avoids the second setup;
  a new batch still pays one setup. Measure that cost before considering a pool
  and its lock-cleanup/discard requirements.
- **Connection budget.** Default four guard sessions plus the application's bounded
  ten-connection pool gives at most fourteen service sessions per process, excluding
  external tools. Startup recovery uses the same admission/coordinator budget.
  Deployment budgets must include every replica; this is not a cluster-wide cap.
- **One batch path.** Pass recovery/admission dependencies into `Bus::start` and
  keep them private. Both routes call the same runner: admit, guard, consume,
  post-sweep, reconcile, close/release. There is no recovery-dependent route branch
  or separate legacy path for tests; tests inject controlled dependencies through
  the same construction and execution interfaces.
- **Post-batch errors.** Attempt reconciliation even when the post-sweep fails.
  Collect both outcomes and log both errors if necessary, then return a sanitized
  retryable `500` for infrastructure failures. Queue successes remain committed;
  no rollback or lost error is implied. Per-saga data errors follow the isolation
  rules above. A pre-sweep infrastructure error prevents consuming a new batch.
- **Recent crashes.** A reservation abandoned less than five minutes ago remains
  pending until a later batch after that timeout. Startup alone is insufficient.
  Continuous overlapping batches can defer exclusive sweeping; let them drain
  and run an empty batch to provide an idle recovery opportunity. No timer is
  introduced in this phase.

### Deployment and session affinity

All consumers sharing these queues must follow the same coordination protocol.
Drain/stop old consumers before switching to this implementation. Do not run old
workers or an uncoordinated consumer beside it; advisory locks cannot protect
work that does not participate. Preparing the module-only baseline is not
permission to reset an existing database or discard retained replay inputs.

Session advisory locks require direct PostgreSQL or session-mode pooling.
Transaction-mode PgBouncer is not a supported guard endpoint. An optional
`COORDINATION_DATABASE_URL` supplies a direct/session-affine endpoint; otherwise
use `DATABASE_URL`. Both must address the same database and queue schema, with
matching search paths. Document this requirement without claiming automatic
proxy-mode detection or substituting transaction locks for batch-long ownership.

### Queue driving

Workers stay off:

- Remove `COMMAND_WORKERS` parsing, the commented-out worker start in
  `src/main.rs`, and the README section about workers. Remove the variable's
  propagation/defaults from the service Makefile, environment examples,
  deployment configuration and tests; do not leave an accepted-but-ignored
  `COMMAND_WORKERS` setting behind.
- A new ADR supersedes [ADR 39](../adr/0039-sourcing-command-workers.md).
- Drive commands and events with `POST /api/commands/run` and
  `POST /api/events/run`. Parallel manual batches are supported.
- Enabling workers later needs recovery that knows about workers, plus a
  periodic sweep and reconcile.

### Replay and retention

- **Retention.** Keep these indefinitely: command rows, triggering and
  canonical result events, saga rows, and the artifacts they reference.
- **Why.** If completed command rows are deleted, a replay can queue work that
  already ran. If canonical events are deleted, commands can no longer be
  rebuilt.
- **Versions.** Keep readers/builders compatible with retained payload versions.
  A fresh-baseline reset does not migrate old fills. An environment that must
  retain replay of old fills needs a separate compatibility/migration plan before
  deployment; draining queues does not waive retention.
- **Tokens.** Expiry and refresh are out of scope. Replay reuses the acquired
  token, and failures follow the ordinary retry rules.

## Delivery

Make one change per commit. For implementation changes, end each phase with `make fmt` and `make ci`
passing. Markdown-only planning edits do not require `make`.

| Phase             | Commits                                                                                                                                                                                                                                                         |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0. Baseline       | Confirm the [starting point](#starting-point).                                                                                                                                                                                                                  |
| 1. Bus            | Add envelope-aware subscriptions (`on_commands_envelope`). Reactions copy correlation and use the event ID as causation. Remove `run_id` from `KeyedCommand`. Add `after_batch` observers. Keep a single batch path in `bus_api`.                               |
| 2. Storage        | Add the saga and queue evidence index migrations, `SagaRecord`, `SagaStatus`, the versioned payload contract, and `SagaRepository`. Run CI on PostgreSQL 18.                                                                                                                            |
| 3. Steps          | Add the `InitializeRetailerFill` step and `validate_retailer`. Remove `run_id` from step payloads. Builders take the namespace and the context snapshot. Move `InitialFillContexts` out of `v2`. Development listings use envelope correlation.                 |
| 4. Saga           | Add the payload model, matching, transitions, listeners and `register_graphql_pipeline`. In one commit, register the saga listeners and remove the step reactions, so exactly one owner dispatches next steps. Add the start endpoint and remove `start_fills`. |
| 5. Reconciliation | Add pending-ID evidence queries, per-saga isolation and log throttling, serialized passes, the startup pass and batch observation.                                                                                                                                                                            |
| 6. Recovery       | Add async admission, advisory-lock recovery and one guard connection per batch, reused for the final sweep. Reconcile despite post-sweep failure. Remove `COMMAND_WORKERS`.                                                                                                                                                                                  |
| 7. Cleanup        | Move pipeline and live tests next to the saga and steps. Delete `src/v2/application`. Drop the parts of the service AGENTS.md "v2 code style" section that the root AGENTS.md "Imports" section already covers.                                                 |
| 8. Documentation  | Write the ADRs, README, Shopify walkthrough, Bruno requests and TODO.                                                                                                                                                                                           |
| 9. Verification   | Run failure/resource-bound tests, query-plan and scale measurements, the PostgreSQL suite and both live fills.                                                                                                                                                                                                                   |

Phase 8 writes three ADRs, numbered at writing time. Each describes only the
final design, with no "remaining work" notes. Old saga ADRs 93/94 are absent from
this baseline, so do not write a new ADR merely to supersede those discarded saga
decisions. The worker-removal decision still supersedes retained ADR 39:

- saga storage and identity;
- saga execution, replay and recovery;
- removal of command workers, superseding ADR 39.

## Verification cases

| Area             | Required evidence                                                                                                                                                                                                                                                        |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Schema           | Migrate and roll back on a fresh database. UUIDs that are not v7, or lack the RFC variant, are rejected. Status and payload checks hold. Both migrations round-trip.                                                                                                     |
| Uniqueness       | Duplicate type/correlation pairs converge on one record. Different types can share a correlation. Initialization with the wrong retailer is rejected.                                                                                                                    |
| Initialization   | The handler never touches saga storage. The first event creates the saga; concurrent and repeated events reuse it. A missing or invalid correlation fails.                                                                                                               |
| Matching         | Unmatched events, including ones with the wrong retailer, change nothing. A matched event with the wrong retailer is rejected. A second saga type on the same correlation (a test-only type) sees only its own work. The same product in two contexts stays two entries. |
| Progress         | Duplicate events, and retried commands with new event IDs, don't inflate counts. Concurrent final events reach a terminal status exactly once.                                                                                                                           |
| Replay           | Replays return the same command IDs and payloads after later transitions and after configuration changes. An unchanged replay doesn't rewrite the row (`xmin` stays the same).                                                                                           |
| Dispatch         | After a failure midway through dispatch, a replay queues only the missing commands.                                                                                                                                                                                      |
| Subscriber order | With `register_graphql_pipeline`, an import subscriber that fails six times leaves the product and the fill `succeeded`.                                                                                                                                                 |
| Reconciliation   | Retryable and exhausted commands and events, a pending published success, successors that were never dispatched, and blocked downstream work each get the right outcome. A malformed saga is counted while healthy sagas keep progressing.                               |
| Recovery         | Stale command/event reservations are swept only with no active batch; recent reservations survive startup and recover on a later empty batch. Exercise two processes sharing the schema, overlapping long batches and eventual recovery after they drain.                                                                |
| Start API        | Mixed valid/unsupported input queues nothing; empty/unknown/duplicate/configuration cases; `202` receipts match IDs/correlation; no storefront HTTP during validation; blocking dispatch off the async thread; partial failures log accepted correlations.                                                                                                                                                               |
| Artifacts        | Development listings use envelope correlation. The same product in two contexts has distinct artifacts and dump paths.                                                                                                                                                   |
| Live             | Anker AU (with token) and Ugreen EU (no token) reach `succeeded` through the API, with parallel batches through the production runner and a second process, respecting configured admission limits. Every structured artifact passes the identity, variant, context and pricing checks.                                                                |

Additional gates before live verification:

- **Admission and connection bounds:** issue 50 overlapping requests with the
  shared limit set to 4, mixing command/event routes. Assert at most four admitted
  batch tasks and guard sessions; waiters hold no thread/connection. Verify one
  dedicated connection setup per batch and reuse for the post-sweep. Check the
  aggregate budget with two processes and rejection of invalid configuration.
- **Cancellation and panics:** cancel while awaiting admission, during blocking
  consumption, and after consumption while observation is running. Guard/permit
  ownership must last until actual work finishes. Panic and connection-error
  paths release resources; subsequent batches progress. A post-sweep failure
  still invokes reconciliation; simultaneous failures are both reported.
- **Reconciler evidence and indexes:** seed substantial unrelated and terminal
  retained history. Verify only pending IDs and relevant trigger/results are
  queried in chunks, canonical outcomes remain authoritative, and one process
  never runs two reconcile passes simultaneously. Use `EXPLAIN (ANALYZE, BUFFERS)`
  on the actual predicates to demonstrate index use; round-trip all indexes.
- **Malformed-state isolation:** unsupported/missing payload versions and corrupt
  progress beside healthy sagas; healthy progress continues, totals stay visible,
  repeated diagnostics are throttled, the cache stays bounded, and a repair is
  picked up by the next pass. Infrastructure failures remain visible.
- **Production parity:** startup, integration and live tests use shared
  registration and runner construction. Query fixtures by explicit saga identity
  and set queue statuses deliberately; no test-only builder or old adapter path
  may stand in for production behavior.

The first implementation's live run produced 152 Anker AU artifacts and 1,234
Ugreen EU artifacts. Catalogs change; those are historical observations, not
expected counts or evidence that the rebuilt implementation passes.

Run these:

- `make fmt` and `make ci` in `services/retailer_sourcing`, and in
  `apps/development` when its listings change.
- `make test-postgres` with `DATABASE_URL_TEST` pointing at PostgreSQL 18.
- `make test-saga-live` with `INITIAL_FILL_LIVE=1`, `DATABASE_URL_TEST`,
  `OBJECT_STORAGE_URL` and `OBJECT_STORAGE_TOKEN` set.
- `PORT=<port> make serve`, then check `/api/health`, both empty batch routes,
  startup recovery, and the intake responses.
- `make audit` only when Cargo dependencies or lockfiles change.
- Benchmark synthetic fills of 100, 1,000 and 10,000 products in one context,
  plus a multi-context case. Report serialized payload size, clone/comparison,
  decoding/encoding time, bytes read/written, lock hold time, catalog fanout and
  reconciliation cost. Measure ordinary transitions separately from catalog replay.
  Verify no full-product command-ID scan, no duplicate match, no serialization
  for change detection, and no no-op update. Include the retained status scan and
  quadratic cumulative payload work; do not claim constant-time transitions.
  Report hardware/environment; avoid arbitrary millisecond pass thresholds.

## Success criteria

A fill is terminal when two things are true:

- every requested catalog has a terminal outcome;
- every known product/context has succeeded, failed or been skipped, with no
  pending work.

A failed catalog means coverage is incomplete, even if every product it did
discover is accounted for.

| Status        | When                                                                                             |
| ------------- | ------------------------------------------------------------------------------------------------ |
| `succeeded`   | Terminal, every requested catalog succeeded, and every product/context reached structuralization |
| `failed`      | Terminal with any failed or skipped catalog or product                                           |
| `in_progress` | Otherwise                                                                                        |

- **Empty catalogs** succeed.
- **Empty queues** never imply success.
- **Skips.** The skip model (a reason plus evidence) stays, but nothing produces
  skips in this phase. Queue errors, HTTP failures and exhausted retries are
  never skips. As a result, one product unpublished during a fill makes the
  whole fill `failed`.
- **Import delivery** is independent. It may continue, or fail, after the
  saga's work is terminal.

## Out of scope

- Token expiry, token refresh, and resolving tokens at execution time.
- Atomic insertion for a request with several retailers.
- Background workers.
- A retention policy shorter than "indefinitely".
- Row-based progress storage, stored command-ID maps/pending sets/counters, and
  pooled guard connections. Revisit storage or pooling in a new decision if the
  measurements justify it; do not introduce either silently during implementation.
- Producing skips.

## Open questions

- Should a verified product disappearance or unpublication count as a skip that
  still allows success? Until that is decided, the strict policy stands.
- Should starting several retailers use one atomic bulk insert?
- `/api/commands/run`, `/api/events/run` and `/api/initial-offer-fills` don't
  require the bearer token; only the development and reporting routes do. This
  predates the saga work. Should they require it?
- Is a progress endpoint needed, for example
  `GET /api/initial-offer-fills/{correlation_id}`, or is inspecting the
  database enough for now?

## Files

| Area        | Files                                                                                                                                                  |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Bus         | `src/platform/bus.rs`, `src/platform/bus_api.rs`, `src/platform/recovery.rs` (new)                                                                     |
| Storage     | `migrations/<date>_sagas/`, `migrations/<date>_queue_correlation_indexes/`, `src/saga/{schema,saga,saga_repository}.rs`                                |
| Steps       | `src/steps/graphql/` (contracts, builders, `initialize_retailer_fill.rs`, `prepare_offer_imports.rs`), `src/steps/prelude.rs`                          |
| Saga        | `src/saga/shopify_graphql_initial_fill/` (`model`, `listener`, `results`, `reconciliation`, `service`, `mod`, `ui/http`)                               |
| Startup     | `src/main.rs`: `register_graphql_pipeline`, the startup sweep and reconcile, and removing workers                                                      |
| Development | `src/development/offer_details_v2/api.rs`, and the dump consumers in `apps/development`                                                                |
| Docs        | New ADRs and `docs/adr/README.md`, the service README, `shopify_graphql_initial_fill.md`, `bruno/retailer_sourcing/initial-offer-fill*.bru`, `TODO.md` |
| CI          | `.github/workflows/ci.yml` (PostgreSQL 18)                                                                                                             |

No Mulac upgrade is needed.
