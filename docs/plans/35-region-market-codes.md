# Region market codes

Status: planned; implementation has not started.

## Goal and current behavior

Use `{"kind":"region","code":"eu"}` for the European Union target market
instead of `{"kind":"region","label":"European Union"}` throughout new
pipeline observations and current API responses. JSON property order is not
part of the contract. Country markets retain their existing country-code rules.

Grounded at monorepo `be253b0`. `lib/retailer_sauce/src/localization.rs` currently
models `TargetMarket::Region { label: String }` and accepts arbitrary nonblank
labels. `lib/retailer_data_extractor/src/localization.rs` emits European Union
from locale region EU. Target markets pass through immutable source exports,
Offers event history, both projections, customer-facing REST/GraphQL and the
customers client. Customer-facing currently maps region labels in
`src/offer_read.rs`; updating only the extractor would leave incompatible readers.

## Contract and compatibility

- Introduce a validated regional code representation, with `eu` as the initial
  supported canonical code. Region codes are distinct from country codes;
  EU must not become an accepted CountryCode. Keep locale parsing and explicit
  evidence precedence unchanged. No URL-based inference or market expansion.
- Accept historical `label: "European Union"` through an explicit legacy decoder
  and map it to `eu`. Preserve old exports, outbox payloads and event bytes.
  Use version-specific DTOs where necessary rather than changing old encoders
  to produce a different historical format.
- Inventory distinct existing regional labels in projections and fixtures before
  migration. Never map `Europe` or another arbitrary label to `eu`, slugify it,
  silently discard it, or invent a code. If non-EU legacy labels exist, retain
  their historical decoding and stop the affected conversion until an explicit
  label-to-code mapping is agreed. Do not mark migration complete while any
  value is unresolved. Unknown new codes and ambiguous label-plus-code inputs
  must fail validation with a useful error.
- Keep null/missing localization behavior and country serialization unchanged.
  Names such as European Union are presentation labels derived from codes,
  not persisted identifiers in the new regional contract.

## Implementation

1. Add a new ADR superseding ADR 33's label-based wire representation and the
   affected localization contract decisions. Leave accepted ADRs intact and
   list the new decision in `docs/adr/README.md`.
2. Add the regional code type/guard and dual-format input adapters in
   retailer_sauce. Update extractor EU fallback and its diagnostics/fixtures.
   Audit all TargetMarket constructors and pattern matches across workspaces.
3. Version affected immutable exports and events when their emitted payload
   changes; allocate the next versions after inspecting current writers.
   Retain existing version readers and replay semantics. Verify Offers import,
   localization-change events, creation events, queued exports and downstream
   inbox readers. Record the exact version matrix in the implementation notes.
4. Add targeted, transactional projection migrations for existing EU region
   objects in Offers and Customer Facing. Update only the regional object,
   preserving language, content, unknown/null facts, IDs, receipts and revision
   metadata. Validate the complete label inventory before writes. Define a
   reversible EU code-to-label down migration; halt on unsupported codes rather
   than losing information. Do not rewrite event stores or immutable artifacts.
5. Update customer-facing's REST/OpenAPI and GraphQL regional fields to carry
   `code` and make the customers client accept both old and new regional inputs.
   Audit other public consumers and document the changed label contract.
   Keep countries, localized content, browsing and offer identity unchanged.
6. Deploy compatible readers first (customers, customer-facing, Offers), then
   coordinate projection migrations with paused projection writers and enable
   new producer versions. Verify backlog delivery and replay before retiring
   legacy compatibility. Rebuilding projections must produce the same codes as
   migrating them in place.

## Acceptance and verification

- Unit tests for canonical EU round trips, legacy EU decoding, country/null
  compatibility and rejection of blank, malformed, unsupported or ambiguous
  regions. Include explicit `Europe` evidence to prove it is not guessed as EU.
- Extraction fixtures for en-EU/eligible Shopify locale fallback and explicit
  market precedence. Preserve evidence paths, original locale and language.
- Historical export/event fixtures remain readable. New versions write only
  code-based EU regions. Verify replay, import retries, unchanged observation
  deduplication and localization-only updates without rewriting identities.
- Isolated PostgreSQL tests for mixed country/EU/null rows, failed preflight on
  unmapped labels, up/down migrations and projection rebuild equivalence.
- REST/GraphQL schema and payload tests, customers old/new input fixtures and
  live pipeline delivery from an EU observation through customer-facing.
- Run `make fmt` then `make ci` for every changed crate/workspace, relevant
  `make test-postgres` targets and live `PORT=<port> make serve` health/page
  checks. Audit only if dependencies change. Record unavailable checks and
  actual checkout revisions; do not reset operational databases.

This task is planning only. Move the TODO from `[>]` to `[~]` when implementation
starts and to `[x]` after implementation and verification are recorded. This work
can proceed independently of plan 36; coordinate their customer-facing migration,
DTO and documentation changes.
