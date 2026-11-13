
### DONE












### IN PROGRESS








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
