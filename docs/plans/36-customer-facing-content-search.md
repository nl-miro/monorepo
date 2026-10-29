# Customer-facing content storage and weighted search

Status: planned; implementation has not started.

## Confirmed scope and current behavior

Drop `title`, `description`, `language`, and `url` from customer-facing's `offers`
table. Store the selected language alongside localized entries in `content`,
and put a URL in each entry with its title and description. Search only the
selected language, weighting title similarity five times description similarity.

Grounded at monorepo `be253b0`. Today `content` is a nonempty JSONB array while
scalar columns duplicate the selected entry. Projection helpers in
`src/offer_projection/content.rs` lock `content, language` and update the list
and scalar tuple together. `src/offer_read.rs` selects those columns, searches
only scalar title through pg_trgm and maps shared REST/GraphQL results.
`UNIQUE (retailer_code, url)` and the title trigram index depend on columns being
removed. The customers client reads a flat title/URL and rejects unknown fields
in shared localized-content entries, so adding a URL requires a compatible
client update. Source/export and Offers event content currently have no per-entry
URL; this task must not silently reinterpret those persisted contracts.

## Storage and public contracts

Use a dedicated customer-facing storage type rather than changing all historical
`retailer_sauce::offer_content::Content` payloads:

```json
{
  "selected_language": "en",
  "entries": [
    {
      "language": "en",
      "title": "Example",
      "description": "Example description",
      "url": "https://retailer.example/product"
    }
  ]
}
```

`selected_language` and entry `language` are required nullable fields. Preserve
exact language keys, with at most one null/unknown entry, unique languages and a
nonempty list. The selection must identify an entry, including null selection.
Keep existing title, description and language validation, deterministic sorting
and URL validation (including byte limits and NUL rejection). Do not infer URLs
from locale names, language tags or translated titles.

- Backfill each existing entry's URL from its offer's existing URL. All these
  entries belong to the same offer, and no historical translated URL is known.
  Preserve selected language exactly and keep every other language intact.
- On creation from existing messages, attach the known creation URL to each
  entry. On content snapshots, preserve URLs for retained language keys. A new
  language without an explicit upstream URL inherits the current selected
  entry's known offer URL; document this compatibility fallback rather than
  claiming a newly discovered localized destination. Legacy language-key moves
  carry the entry's URL with its title/description. Do not invent an upstream
  URL-change event as part of this storage refactor.
- Keep public `content` as an array, adding `url` to its entries; the storage
  envelope is private. Continue returning flat `title`, `url` and
  `localization.language`, derived from the selected entry/envelope. Retain the
  existing description exposure. Cards and offer links use the same selection.
- Add a compatible customers wire adapter for old content arrays without URLs
  (fallback to flat offer URL), new arrays with URLs, and existing legacy
  absent/object content. Preserve validation of selected flat title/URL and
  avoid relaxing unrelated unknown-field rules in immutable event DTOs.

## Search definition

Keep query normalization, the 1–200 character rule, pagination, REST/GraphQL
parity, parameter binding and transaction-local three-second statement timeout.
Use only the selected entry's title and description. An unselected translation
must not make an offer match or affect ranking.

Using the existing schema-qualified pg_trgm `word_similarity` and lowercase
inputs, define `t = word_similarity(query, title)` and
`d = word_similarity(query, description)`, with absent/empty description scoring
zero. Candidate eligibility is the existing 0.5 word-similarity threshold on
**either** title or description; retain the transaction-local setting. Rank
eligible offers by `5 * t + d` descending, then creation time descending and UUID
descending. Do not put exact-match status ahead of the weighted score, which
would override the requested weights. Keep matching and scoring separate so a
weak title does not qualify merely because it is multiplied by five.

Search the stored text consistently with current title semantics; do not add
translation, stemming or HTML rewriting. Preserve browse ordering and one extra
row for `has_more`; avoid duplicate rows or inflated image lists. Update the
customers search prompt to mention title or description.

## Implementation and rollout

1. Add an ADR superseding ADR 38's duplicated scalar storage for customer-facing
   and the affected title-only search decision. Keep Offers' storage and source
   contracts unchanged. Record selected-language search, storage shape, URL
   compatibility behavior and public API compatibility.
2. Implement typed conversion/validation and selection helpers. Refactor every
   creation, full-content, historical title/description/localization handler to
   write only the new envelope atomically. Retain row locks and message receipt
   transaction boundaries. Market-only changes must not alter content/selection.
3. Add a transactional migration after the current localized-content migration.
   Validate existing lists and scalar/selected-entry agreement before destructive
   steps; fail on inconsistencies rather than dropping divergent data. Build the
   envelope and copy the current URL to every entry before dropping all four
   scalar columns. Preserve IDs, prices, images, source timestamps, target
   markets, receipts and creation times.
4. Replace the old URL uniqueness constraint with equivalent uniqueness on
   retailer code plus selected content URL. Keep UUID as entity identity and
   prove collision rejection, including selection changes. Preserve the old
   URL length invariant per entry through application and database validation.
5. Introduce schema-qualified immutable SQL extraction helpers for selected title,
   description and URL (matching the Rust null/selection rules exactly). Build
   trigram expression indexes for selected title and description and a unique
   selected-URL expression index. Use those same expressions in queries and
   bind values. Drop the old title index only when the replacement is ready.
   Do not change an immutable indexed function's meaning without rebuilding
   dependent indexes. Keep pg_trgm in `customer_facing_extensions` and retain
   existing trusted-schema requirements.
6. Update read records, browse/search SQL, DTO mapping and customer client input
   types. Add URL to REST/OpenAPI and GraphQL content entries while deriving flat
   compatibility fields from the selected entry. Do not expose the storage
   envelope or add duplicated authoritative scalar columns.
7. Define a down migration that reconstructs the scalar selected tuple and old
   content array/indexes/constraints. Explicitly document that old storage cannot
   retain distinct URLs for unselected languages: refuse lossy downgrade when
   such URLs exist, requiring backup/recovery instead. Test up/down/up for the
   existing shared-URL data without loss.
8. Deploy the compatible customers reader first. Pause customer-facing projection
   writers and reads during its coordinated migration/binary switch; the old
   service cannot run on the dropped-column schema. Resume and replay pending
   messages normally. No queue resets, receipt deletion, history rewrite or
   unrelated Offers database migration. Update affected READMEs and API examples.

## Acceptance and verification

- Unit tests for null/known selection, invalid selections, duplicate languages,
  empty lists, preserved descriptions, per-entry URL validation, old/new client
  formats and consistent flat compatibility fields.
- Isolated migration fixtures with multilingual content, null selection, missing
  descriptions and boundary URLs. Assert all four columns are absent after up;
  verify every retained fact, migrated constraints, rollback refusal on distinct
  URLs, and recoverable down/up for compatible data.
- Replay legacy creation/title/description/language changes and new full content
  snapshots against the new store. Check preservation of URLs by language,
  new-language fallback, selection-only changes, receipt idempotence, row locking
  and malformed-message rollback. Compare migrated and rebuilt projections.
- Search fixtures for title-only and description-only matches, typos, missing
  descriptions, non-selected language exclusion, selection changes, and evidence
  that an equal similarity increase contributes five times as much through title
  as through description in the documented score. Test ties and stable pagination; REST and GraphQL
  return the same offers and retain ordered images and metadata.
- Run `EXPLAIN (ANALYZE, BUFFERS)` on representative isolated data for both search
  predicates and combined ranking, including broad queries and long descriptions.
  Verify index usability, no missed qualifying matches and acceptable performance
  under the existing timeout. Avoid per-request full JSON expansion across all
  translations; only extract the selected entry. Prove SQL/Rust selection parity.
- Run `make fmt` then `make ci` in customer-facing, customers and any changed
  shared crate; run customer-facing `make test-postgres` and new migration/search
  tests explicitly. Audit only if dependencies change. Start both apps with
  separate `PORT` values using `make serve`; check health, browse/search, chosen
  titles/URLs and customers page behavior. Record actual results and blockers.

This is a plan only; finish at `[>]`. Plans 35 and 36 can be implemented
independently, but coordinate customer-facing DTOs, migration numbers and the
combined client compatibility fixtures. Do not change existing accepted ADRs.
