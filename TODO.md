
### DONE












### IN PROGRESS

- [~] Update money to at least 0.1.2 in every consuming workspace and validate.

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
