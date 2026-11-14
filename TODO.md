- [~] Drop all_markets and the discover-markets stage.
  Fan out entity contexts after token acquisition; stub the published trait methods.

- [x] Remove redundant per-retailer request and validation wrappers.
  All 48 configurations use direct Integration calls in tests; imports are
  test-only where appropriate. Formatting and all three brand CI checks pass.

- [x] Remove the unused Markets GraphQL query from all three brands.
  Removed query/parser/exports; the required legacy trait method returns an
  error without HTTP. All three brand CI checks and sourcing initial-fill tests pass.

- [x] Restore acquire_token to true for all disabled stores.
  Updated tests and documentation; affected library and sourcing CI pass,
  and the service health endpoint returns HTTP 200.

- [x] Disable token acquisition for ugreenin, ugreenuk and ugreennl.
  Updated policy and historical-token checks; Ugreen and sourcing CI pass,
  and the service health endpoint returns HTTP 200.

- [x] Disable Storefront token acquisition for the five affected retailers.
  Flags are honored in discovery/HTTP and API validation, including queued-token
  retries. Affected crate CI and all five PostgreSQL workflow tests pass; the
  service health endpoint returns HTTP 200.

- [x] Preserve actionable Shopify GraphQL errors in initial fill.
  All three brands retain operation, messages, codes, paths, locations and scope/cost
  diagnostics before typed decoding; missing data is distinct and tokens are redacted.
  Library CI checks pass (final crate retried without sccache after a cache reset);
  sourcing initial-fill tests pass.

- [x] Use Retailer supported locales as initial-fill discovered markets.
  Main-table locales only; optional country and response-derived currency. ADR 69.
  Library, sourcing and app CI pass; five PostgreSQL fixture tests cover all 48
  integrations. Sourcing health and development page return 200. Audits show no
  new advisories; yanked-crate checks are limited by the read-only registry cache.
  Live Shopify fills remain a deployment acceptance check.
- [x] Restrict retailer entity locales to main inventory tables.
  Common lookup and locale accessor added; retailers_wip make ci passes.


### DONE












### IN PROGRESS

- [x] Add and use Retailer::new.
  All 50 retailer definitions use the constructor; formatting and crate CI pass.

- [x] Add and use SupportedLocales::new.
  Constructor used throughout retailer definitions and inventory tests; crate formatting and CI pass.

- [x] Replace SupportedLocales annotation fields with construction comments.
  Evidence notes preserved beside definitions; inventory checks and crate CI pass.

- [x] Rename Locale variants to language_REGION notation.
  All 106 variants and their uses updated; canonical tags unchanged. Crate CI passes.

- [x] Use the shared RetailerCode enum in WIP retailer definitions.
  All 50 definitions are typed; crate CI passes. Audit reports no new advisories.

- [x] Represent inventory locale tags with a canonical Locale enum.
  All factories use 106 variants; casing aliases normalize. Crate CI passes.

- [x] Use country mapping only for shipping lists longer than four entries.
  Shorter lists use direct Market values; crate CI passes.

- [x] Simplify shipping-country lists by mapping Country arrays into Market values.
  All brand factories updated; crate CI passes.

- [x] Represent retailer countries and regions with Market, Country and Region enums.
  Factories use typed markets and locale contexts; code conversion and inventory checks pass.

- [x] Use plain vectors for retailer target markets and shipping countries.
  Unknown values use empty vectors; factories and checks updated. Crate CI passes.

- [x] Remove target-market annotations, shipping restrictions and configured shipping countries from retailer entities.
  Factories, inventory tables and checks updated; research evidence retained. Crate CI passes.

- [x] Verify shipping-country configuration for every retailer using shop.shipsToCountries.
  Saved 48 lists across 50 entries; Vietnam is locked and Korea has no Shopify endpoint.
  Configuration is separate from documented coverage; crate CI passes.

- [x] Correct Anker EU shipping destinations from storefront and GraphQL shipping configuration.
  Both sources report 24 countries; evidence saved and WIP factory synchronized. Crate CI passes.

- [x] Make catalog and product GraphQL country optional across all three brands.
  Missing market context omits country; explicit context checks remain. Library CI passes.

- [x] Prefix locale context column headers with GraphQL.
  Crate formatting and CI pass.

- [x] Remove redundant advertised-only annotations from locale table rows.
  Inventory checks infer annotations from headers; crate CI passes.

- [x] Use locale table column headers instead of separate section labels.
  Crate formatting and CI pass.

- [x] Split retailer locale tables into main and advertised-only entries.
  Inventory consistency checks account for grouping; crate CI passes.

- [~] Verify advertised retailer locale contexts against live storefront evidence.
  Eight contexts verified and synchronized; crate CI passes. Eight language pages
  still need country policy, one country mismatch remains, and 113 rows are
  unresolved after HTTP 429/network failures. Evidence: docs/research/retailer-context-verification-2026-11-13.json.

- [x] Add missing split Nordic retailers to the inventory and WIP factories.
  All five country contexts and legacy compatibility covered; crate CI passes.

- [x] Model retailers_wip inventory and brand factories from docs/retailer.md.
  Includes bidirectional documentation links and inventory consistency tests; crate CI passes.

- [x] Add retailers_wip retailer modules and empty Retailer/SupportedLocales entities.
  Crate formatting and CI pass.

- [~] Enable all advertised market contexts for all 17 Anker initial fills.

- [x] Move product SEO title/description into content with seo_ prefixes across all brands.
  Source values/nulls are preserved; lib CI passes.

- [x] Reject variant compare-at currency mismatches across all three brands.
  Includes zero-price variants; lib CI passes.

- [x] Replace extracted checkout MoneyV2 with money::Money across all brands.
  Validates currencies, precision and overflow; lib CI passes.

- [x] Recover missing Anker context from recorded response and price evidence.
  Lib CI passes. Temporary corpus run writes 9,194 artifacts; 305 retain missing-media-ID errors.
  Requested extraction into ../data was attempted but writes are blocked by its read-only mount.

- [x] Require acquired_at and context in all three extracted models.
  Missing/null metadata and malformed timestamps are rejected; lib CI passes.

- [x] Update money to 0.1.2 in every consuming workspace and validate.
  All workspace CI passes; audits report no new advisories. Customers and sourcing
  HTTP checks pass; offers/customer-facing startup requires DATABASE_URL.

- [~] Deduplicate extracted image IDs across all three brands; reject conflicting records.
- [~] Repair extraction contract tests and verify library CI.








### TODO








### BACKLOG






### ARCHIVE

- [x] Add extracted/graphql.rs modules declared from extracted.rs across all three brands.
  Formatting and Clippy pass; lib CI retains the three extractor contract failures.

- [x] Move commented GraphQL artifact types and converters into extracted/graphql.rs across all brands.
  All nine requested types and their converters remain commented out. Formatting
  and Clippy pass; lib CI retains the three extractor contract failures.

- [x] Move unused extracted Connection, Edge and PageInfo types and converters into graphql.
  Types and converters are commented out across all three brands. Formatting
  and Clippy pass; lib CI retains the three extractor contract failures.

- [x] Move commented PriceRange and Count types and converters into extracted/not_interested.rs.
  Module declared in each brand’s extracted.rs; all moved code remains commented.
  Formatting and Clippy pass; lib CI retains the three extractor contract failures.

- [x] Repeat three unused extracted-type passes and run v2 extraction against ../data.
  Pass totals: 3 (IsoCode in each brand), 0, 0. Definitions and converters commented.
  Formatting and Clippy pass; lib CI retains the three extractor contract failures.
  Extraction built and ran but could not write to read-only ../data.

- [x] Move Product and its identifiers, taxonomy and content types into extracted/product.rs across all brands.
  Re-exports preserve existing public paths. Formatting and Clippy pass;
  lib CI retains the three extractor contract failures.

- [x] Move product-exclusive extracted types and their dependencies into product.rs across all brands.
  Moved 33 Anker types and 31 each for Minisforum/Ugreen, preserving public
  paths via re-exports. Formatting and Clippy pass; lib CI retains the three extractor contract failures.

- [x] Move commented IsoCode and its converter into extracted/graphql.rs across all brands.
  Formatting and Clippy pass; lib CI retains the three extractor contract failures.

- [x] Add collection, image and media modules under extracted across all brands.
  Module files created and declared in extracted.rs. Formatting and Clippy pass;
  lib CI retains the three extractor contract failures.

- [x] Move Collection and its exclusive ProductSummary type into collection.rs across all brands.

- [x] Move Media and its exclusive helper types into media.rs across all brands.
  Collection/ProductSummary and Media/MediaPresentation/VideoSource/ModelSource
  now live in their modules; re-exports preserve public paths. Formatting and
  Clippy pass; lib CI retains the three extractor contract failures.

- [x] Expose extracted types through prelude modules and update consumers across all brands.
  Type re-exports now live in extracted/prelude.rs; converters, tests and storefront
  exports use the prelude. Formatting and Clippy pass; lib CI retains the three extractor contract failures.

- [x] Add a Language enum for GraphQL language codes in retailer definitions.
  All brand definitions use typed languages; formatting and crate CI pass.
