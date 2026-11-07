
GOAL: refactoring offers aggregate into more familiar Br4oadway style

[x] Fix compilation against the current Mulac event repository API in customer-facing and retailer-sourcing tests. Use DieselPgEventRepository directly and supply the test repository parameter to EventingService. Root make check and services/apps all-target checks passed. Customer-facing CI passed (44 tests); sourcing formatting/lint and all six bus API tests passed. Full CI hit an unrelated library doctest linker crash and sandbox socket restrictions in sourcing tests. Startup checks were blocked by missing customer-facing DATABASE_URL and sandbox socket restrictions.

[x] Split the export-message listener's Store into ExportStore and OutboxStore traits, implemented using the existing object-storage client and PostgreSQL respectively. Retain the export's PostgreSQL metadata lookup. [Implementation plan](docs/plans/38-export-loader-and-outbox-recorder.md).


[x] Create an OutboxRepository trait and PostgreSQL implementation for recording export messages. `make fmt` and initial service `make ci` passed; the final CI rerun and live startup were blocked by sandbox socket permissions.

[x] Extract the outbox record INSERT SQL into a local variable. Formatting and lint passed; CI socket tests and live startup were blocked by sandbox permissions.

[x] Extract the outbox record SELECT SQL into a local binding. Formatting and lint passed; CI socket tests and live startup were blocked by sandbox permissions.

[x] Comment each stage of platform outbox record. Formatting and lint passed; CI socket tests and live startup were blocked by sandbox permissions.

[x] Report all missing outbox message IDs in a batch error. All four outbox unit tests, formatting and lint passed; full CI and live startup were blocked by sandbox socket permissions.

[x] Move outbox metadata conversion onto OutgoingMessage and replace the standalone helper. All four outbox tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Format the outbox INSERT SQL for readability and label its parameters. Formatting and lint passed; CI socket tests and live startup were blocked by sandbox permissions.

[x] Propagate outbox metadata serialization failures instead of panicking. All four outbox tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Enclose the outbox INSERT SELECT block in parentheses for readability. Formatting and lint passed; CI socket tests and live startup were blocked by sandbox permissions.

[x] Remove inline SQL comments and tidy the parenthesized outbox INSERT query. Formatting and lint passed; CI socket tests and live startup were blocked by sandbox permissions.

[x] Extract grab_unique_messages before building outbox payloads. All four outbox tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Extract outbox insert-result loading and validation into a helper. All four outbox tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Add a separate inherent implementation block for PgOutboxRepository. Formatting and lint passed; CI socket tests and startup were blocked by sandbox permissions.

[x] Scaffold PgOutboxRepository record with input generation, insertion and result-checking stubs. The stages intentionally use todo! and recording is not operational yet. Formatting, lint and four legacy outbox tests passed; full CI and startup were blocked by sandbox socket permissions.

[x] Copy the platform outbox INSERT SQL and execution into PgOutboxRepository::insert_inputs. Added aligned ID, payload and metadata inputs; generation and result verification remain stubs. Formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Use OutboxEntryRecord values for a Diesel batch insert instead of parallel arrays and UNNEST. Preserves ID ordering and ON CONFLICT DO NOTHING. Formatting and lint passed; full CI and startup were blocked by sandbox socket permissions. Input generation and result checking remain stubs.

[x] Implement PgOutboxRepository input generation with validation and delivery metadata. Three focused tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions. Result checking remains stubbed.

[x] Replace OutboxInputs with a vector of OutboxEntryRecord values. Generation returns Vec; insertion and checking borrow slices. Three repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Rename PgOutboxRepository::generate_inputs to generate_records. Three repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Rename PgOutboxRepository::insert_inputs to insert_records. Three repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Pass the sorted record slice directly to Diesel in insert_records. Renamed the argument to records. Three repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Use validate_payload returning unit for outbox record generation. Three repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Implement validate_payload directly without calling parse_object. Per follow-up, accepts any valid JSON and errors only on parsing failure. Four repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Bind message metadata before serializing it in generate_records. Four repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Bind each OutboxEntryRecord before pushing it onto records. Four repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Bind id, status and payload before constructing each outbox entry record. Four repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Add and use OutboxEntryRecord::new. Four repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Return immediately for an empty PgOutboxRepository batch. Four existing repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions. Result checking remains stubbed; transaction behavior was not verified against PostgreSQL.

[x] Run repository insertion and result checking on one connection in a transaction. Four existing repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions. Result checking remains stubbed; transaction behavior was not verified against PostgreSQL.

[x] Create the platform outbox folder with error, repository and record modules. Existing implementation is in mod.rs; new modules are empty scaffolds. Four outbox tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Move OutgoingMessage and its metadata method into platform outbox model. Re-exported from outbox for existing callers. Four outbox tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Add a schema module to platform outbox. Empty schema.rs registered in mod.rs. Formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Rename OutboxRepository::record to bulk_insert and update its callers. Ten listener/repository tests, formatting and lint passed; full CI and startup were blocked by sandbox socket permissions.

[x] Use sibling Mulac path dependencies across all consuming crates, update CI/docs and verify resolution. Locked offline metadata confirms all 12 Mulac packages resolve locally; unrelated packages are unchanged. Customer-facing CI passed (44 tests). Offers lint passed and 100 tests passed; four socket tests were blocked by sandbox permissions. Sourcing CI/startup remain blocked by the ongoing outbox refactor (entry-type mismatch, missing constructor and vector conversion). Offers/customer-facing startup requires database configuration. Fresh audit fetch was blocked; cached audit has the same nine vulnerabilities and five warnings as the prior lockfile, with no new findings. [ADR 45](docs/adr/0045-local-mulac-path-dependencies.md).

[x] Update Mulac imports from io to prelude across consuming services and their tests/docs. Formatting passed; customer-facing CI passed (44 tests) and offers CI passed (104 tests). Sourcing CI/startup remain blocked by the existing outbox refactor errors; offers/customer-facing startup requires DATABASE_URL.

[~] Migrate Mulac dependencies, imports and usages to the current sibling API. Replace removed API/adapter crates with feature-enabled domain crates, migrate command storage and outbox entry construction, then run CI, audit and startup checks.

BACKLOG
 - Enforce that there is an empty line before any implicit or explicit return


Status follows the [TODO workflow](AGENTS.md#todo-workflow):

| Marker | Phase                      |
| ------ | -------------------------- |
| `[ ]`  | Pending                    |
| `[.]`  | Planning in progress       |
| `[>]`  | Planning complete          |
| `[~]`  | Implementation in progress |
| `[x]`  | Implemented                |

Flow: `[ ]` → `[.]` → `[>]` → `[~]` → `[x]`.

NOTES:
- We are aware that `For now` is going against ADR-16, but this is shortterm
- depending on local mulac will be needed in this phase and possibly next ones
- `customer-facing` will take given events and persist them into `offers` table


BACKLOG: leave this alone
- Replace offer images stored as `Vec<String>` with a custom collection that removes duplicates during construction.
- Determine item condifition from title (refusrbished ...)
- Refactor how offer duplication is handled: use gtin or sku per retailer?
- Refactor commanding by introducing priority column, first command starts with priority 100 and every after it does -1 on current priority. If we reach 10, we stop with -1 and use last one. That way we fast track first batch to finish point. When reserving commands, it should also order by priority asc
- Drop descripton column on `customer_facing`'s 'offer' table
