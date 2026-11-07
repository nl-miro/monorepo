# Separate export loading from outbox recording

Status: implemented and verified on 2026-11-03.

Grounded on monorepo `b21831fdd03d6e60e5e03f2051538a69549a0654` on
2026-11-03. Tracks the separate item in [TODO.md](../../TODO.md).

## Outcome

`RecordOfferExportMessages` receives two independently replaceable dependencies:
an `ExportStore` and an `OutboxStore`. Each trait exposes only its own
operation. The listener continues to load the pinned export, validate it,
serialize one message per entry and record the batch with the event context.

The `ExportStore` implementation uses the existing object-storage client to
retrieve export bodies. The `OutboxStore` implementation uses PostgreSQL to
record messages. They have separate implementations and errors; the listener
depends on the two traits. Export bodies remain in object storage and outbox
messages remain in PostgreSQL.

## Export-reference decision

The existing client retrieves an object by its storage record ID. The event's
`export_id` is the export's idempotency key, a different UUID. Currently the
completed `offer_export` row supplies the storage record ID and expected size.
The event's `source.storage` describes the upstream structuralized input, not
the completed export object, so it cannot be reused to fetch the export.

The user confirmed retaining the PostgreSQL metadata lookup. The export adapter
therefore receives the database pool and existing object-storage client, loads
the completed row by export ID, and uses its reference to fetch the body. Reuse
the existing discovery export reader to preserve all validation. The event shape
and historical queued events remain compatible; no new object-storage endpoint
or migration is required.

## Implementation before the split

- [listener.rs](../../services/retailer_sourcing/src/offer_delivery/record_export_messages/listener.rs)
  defines `Store` with both `load` and `record`; `Listener<S>` owns one store.
- The former feature `store.rs` implements both through `PgStore`, which holds
  an `ExportStore` and a database
  pool. Its `StoreError` combines loading, connection and recording failures.
- Loading delegates to `ExportStore::load`: it reads a completed `offer_export`
  row and fetches its immutable body from object storage.
- Recording delegates to `platform::outbox::record`, which writes
  `outbox_entries` as one atomic batch. The load is outside that transaction.
- Registration constructs the combined store; unit tests use one `FakeStore`,
  and PostgreSQL tests construct `Listener<PgStore>` directly.

## Implementation steps

1. Replace the combined trait with feature-local `ExportStore` and
   `OutboxStore` traits, retaining `Send + Sync + 'static` bounds:
   - `ExportStore::load(&self, id: Uuid) -> Result<Option<OfferExport>, ExportLoadError>`.
   - `OutboxStore::record(&self, messages: &[OutgoingMessage], context: &EventContext)
     -> Result<(), OutboxRecordingError>`.
   Keep the traits in `listener.rs`, matching the slice's current convention.
   Alias the existing discovery export error as `ExportLoadError`, preserving
   its database, storage and validation causes. Define `OutboxRecordingError`
   with pool-acquisition and `platform::outbox::RecordError` variants. Avoid a
   shared error enum that reconnects the two responsibilities.
2. Change the listener to `Listener<L, R>` with `exports: L` and `outbox: R`,
   injected by `Listener::new(exports, outbox)`. Keep source/schema validation
   and serialization in the listener. Add separate load and recording variants
   to `ListenerError`, preserving underlying causes and event retry behavior.
3. Add `export_store.rs` with `ObjectStorageExportStore` implementing the new
   `ExportStore` trait. Wrap the existing discovery export reader, constructed
   with the database pool and existing object-storage client, and delegate its
   `load` operation. Alias that imported concrete reader as `PreparedExportStore`
   to distinguish it from the new trait. This retains the completed-row lookup,
   object retrieval, deserialization and validation without copying their logic.
4. Add `outbox_store.rs` with `PgOutboxStore`, holding only `DbPool`.
   Implement `OutboxStore` by acquiring a connection and delegating the full
   batch to the existing `platform::outbox::record`. Keep SQL and transaction
   ownership in that existing recording function.
5. Update `prelude::register` to construct `ObjectStorageExportStore` with the
   pool and object-storage client, construct `PgOutboxStore` with the pool, and
   inject both. Update module declarations and remove the combined `store.rs`
   and `StoreError` after migrating their callers.
6. Split unit-test `FakeStore` into independently configurable `FakeExportStore`
   and `FakeOutboxStore`; adapt PostgreSQL helper signatures and error
   assertions. Preserve existing integration fixtures and behavioral coverage.
   Add meaningful assertions that the recorder is never called after a missing
   export, load failure or validation failure, and that a recording failure
   propagates after successful loading. Verify the exact requested export ID
   and that the recorder receives the complete batch and unchanged context.
7. Update the sourcing README's recording section to explain the two
   dependencies and their separate transaction boundary. Observe the repository
   rule requiring a blank line before explicit and implicit returns in edited
   code. Keep unrelated formatting changes separate.

## Preserved guarantees

- Only the event's completed snapshot is loaded; no latest-source fallback.
- Entry UUIDs, bare import payloads, routing key `offers`, event type
  `OfferExportWasPrepared` and envelope metadata stay unchanged.
- Exclusions generate no messages; empty exports succeed with an empty batch.
- Recording remains atomic, conflict-aware and idempotent. Replays and concurrent
  notifications preserve existing payloads, metadata and delivery lifecycle state.
- Failures still reach the event runner and its existing retry policy.
- Export loading, outbox recording and event acknowledgement remain separate
  operations; introducing two traits does not add a cross-operation transaction.
- Delivery still happens through the separate outbox runner.

## Acceptance and verification

- Production registration and tests inject separate export and outbox stores;
  no combined feature `Store` trait or adapter remains.
- The export adapter uses the completed PostgreSQL metadata row and existing
  object-storage client; the outbox adapter holds only its PostgreSQL pool.
- Existing listener mapping, failure, empty-export and exclusion tests pass.
- Existing PostgreSQL replay, concurrency, conflict rollback, delayed-event,
  metadata and delivery tests pass with the split dependencies.
- Run `make fmt` followed by `make ci` in `services/retailer_sourcing`.
- Run `DATABASE_URL_TEST=<disposable database> make test-postgres` there with
  the suite's existing object-storage fixtures. Preserve smoke coverage; no new
  route needs additional smoke tests.
- Start the service with `PORT=<unused port> COMMAND_WORKERS=0 make serve` and
  verify `/api/health`; use controlled configuration for any database-backed run.
- Audit is required only if implementation unexpectedly changes Cargo
  dependencies or lockfiles. Record actual checks and any unavailable integration
  prerequisites before advancing the TODO to implemented.

## Implementation and verification

The listener now accepts independent `ExportStore` and `OutboxStore` traits.
`ObjectStorageExportStore` wraps the existing discovery reader, retaining its
completed-row lookup, immutable object fetch and validation. `PgOutboxStore`
owns only a database pool and delegates atomic recording to the existing outbox
function. The combined feature store was removed. Loading and recording have
distinct errors and independent test doubles; tests check the requested export
ID, full message batch and event context, plus failure short-circuiting.

Integration testing exposed existing fixtures that lost `scraped_at` when
replacing storage metadata, and a mixed-variation fixture missing localization.
The fixtures now retain their timestamp and supply a target market/language so
the tests exercise export recording rather than fail during preparation. No
production validation was weakened.

The earlier planning CI failure came from an empty leftover directory named
`2026-11-03-000001_export_database_payload` under migrations. Removing that empty
directory restored compilation; no migration file or database schema changed.

Verification against the working tree based on the revision recorded above:

- `make fmt` then `make ci` in `services/retailer_sourcing` passed: 161 tests
  passed across targets, with 79 opt-in tests ignored. Socket-using tests ran
  outside the sandbox after the initial restricted run failed to bind sockets.
- `DATABASE_URL_TEST=postgresql://postgres@127.0.0.1:55432/export_stores_test
  RUST_TEST_THREADS=4 make test-postgres` passed all 70 tests against a dedicated
  disposable database with isolated schemas and local object-storage fixtures.
  This includes replay, concurrency, conflict rollback, pinned snapshots,
  envelope metadata and HTTP inbox acceptance.
- `PORT=3468 DATABASE_URL= COMMAND_WORKERS=0 make serve` started successfully;
  `GET /api/health` returned HTTP 200 and `ok`. The temporary server was stopped
  after the check. This live check ran without persistence; database behavior
  was verified by the integration suite.
- `git diff --check` passed. No Cargo dependencies or lockfiles changed, so no
  audit was required. External object-storage live tests were not run; this
  refactor reuses the existing client and was tested against local fixtures.
