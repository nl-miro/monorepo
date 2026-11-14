# retailers_anker

Integration for the Anker retailer.

The former Nordic integration now has five active market codes: `ankerse`
(Sweden), `ankerno` (Norway), `ankerfi` (Finland), `ankeris` (Iceland), and
`ankerdk` (Denmark). They share the Nordic sitemap and product URLs, but fetch
country-specific Storefront API prices in SEK, NOK, EUR, ISK and DKK.
The Swedish market here is the existing Nordic store, not `anker.com/se`.
`ankernordicscom` remains available for historical records and queued work.

For these five codes, `retailer_data_extractor::extract_for` reads the downloaded
API JSON directly. Sourcing uses `offer_request_for` to build the country query;
configure `ANKER_NORDICS_STOREFRONT_TOKEN` with the site's public browser
Storefront token. Raw offline exports can retain `.html` filenames but contain
JSON. HTML alone is rejected because it carries Swedish-default prices.
Market, currency, handle, GraphQL errors and incomplete responses are checked
before storage. Products needing more than 250 variants/images fail explicitly.
See [ADR 47](../../docs/adr/0047-nordic-retailer-markets.md) for compatibility
and rollout. Each Nordic retailer owns its legacy query, raw types, parser and captured
response in `offer_details/v1/graphql`. `nordic` only routes compatibility calls
and retains the existing normalized document format.

Each storefront has an offer details pipeline in `offer_details/v1`:

- `architecture()` extracts the page's data islands: the `ld+json` scripts
  plus, depending on the storefront, the Next.js `__NEXT_DATA__` script, the
  App Router flight payload, or the Liquid `ProductJson` record.
- `normalize()` reads them into a strict raw schema, which rejects unknown
  fields, and then into a normalized document.
- `structuralize()` turns the normalized document into the structuralized one.

`ankerkr` runs on Cafe24, has no data island to read, and produces no document
yet.

The AU and US normalizers skip product pages whose URLs identify another known
Anker storefront; older crawls placed regional pages in these folders. Missing
payloads on native pages still fail. German products can omit their featured
image, and EU/Polish specification rows can have text with no separate label
(`name: null`). Japanese PowerBag records without Product JSON-LD retain their
Liquid variants in JPY and use their storefront handle for offer URLs. Italian
JSON-LD offer prices retain their JSON number representation so decimal prices
also deserialize when object-storage clients enable `arbitrary_precision`.
Canadian and German structuralized variations retain Shopify identifiers when
their merchant SKU is blank, with `sku: null` instead of rejecting the page.

The `ankerau` tests run the pipeline over saved product pages in
`../data/ankerau`, next to the monorepo checkout, and skip when that folder is
missing. They select `au-products-*.html`, matching the URL-path filenames
written by the export tool, because older crawls also saved other regions in
that folder. The other storefronts are covered by unit tests. The facade's
`normalization_report` integration test also checks saved extracted pages from
the storefronts with previously reported normalization failures.

`ankerau` is the reference implementation in the
[retailer integration playbook](../../docs/retailer-integration.md), which
describes how to add or finish a storefront.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

To exercise the Italian regression under the development app's JSON features:

```sh
cargo test -p retailers_anker ankeritalycom --features serde_json/arbitrary_precision
```

The Italian corpus test reads `../data/ankeritalycom/offer_details/extracted`,
checks normalization and structuralization without writing files, and skips when
that data checkout is absent.

Canonical comparison prices, availability, item condition, and descriptions are
documented in [offer facts source coverage](../../docs/offer-facts-source-coverage.md).

## Anker initial fill

The versioned AU/CA integrations and shared brand engine support all 17 active
Anker stores except Korea. Shared typed
queries take country/language variables for every operation and continuation.
Collection contexts come from the main-table inventory locales, including
English and French for Canada.
Nordic markets keep separate prices and discover their browser token automatically
for initial fill; `ANKER_NORDICS_STOREFRONT_TOKEN` remains an optional override
(and is still required by the separate sitemap offer-download flow). See
[ADR 54](../../docs/adr/0054-anker-browser-token-discovery.md).
Italy/Japan attempt optional public token acquisition and retain token-free access
when no token is found; other stores
acquire the selected browser token from their configured source. The full field
selection and exhaustive pagination are preserved for both access modes.
All retailers use initial pages of 5 for every selected connection. Continuations
fetch 25 collections/images/media, 10 variants, and 5 items per nested variant
connection. Authentication does not change these sizes; see
[ADR 56](../../docs/adr/0056-anker-operation-specific-page-sizes.md).
Selling plans are not selected; see
[ADR 73](../../docs/adr/0073-drop-storefront-selling-plans.md).
See the [market table](../../services/retailer_sourcing/README.md#anker-initial-offer-fill),
[ADR 53](../../docs/adr/0053-anker-storefront-initial-fill-markets.md), and
[captured evidence](src/shopify/initial_fill/fixtures/README.md).

The checked-in schema and full details selection come from
`wip/switching-shopify-retailers-to-api` (`c3f5872`). Catalog requests select only
product GIDs and cursors. Separate typed continuation operations cover product
and variant connections. The saved WIP response in the fixtures
has incomplete collections and is deliberately retained as a pagination regression.
Presentation selections additionally request `id` to avoid the generated client's
empty enum for objects selecting only `__typename`.
`make ci` compiles the query documents and tests response validation; no live
Shopify request is made by the library tests.

## Offer details v2 raw representations

AU and CA expose `AnkerAuRawProduct` / `AnkerCaRawProduct`,
`AnkerAuRawResponse` / `AnkerCaRawResponse`, and
`AnkerAuRawOfferDetails` / `AnkerCaRawOfferDetails` from `prelude`.
These deserialize a merged product, a GraphQL response (single product or the
older full-products connection), and a stored initial-fill artifact respectively:

```rust
use retailers_anker::prelude::AnkerAuRawOfferDetails;
let raw: AnkerAuRawOfferDetails = serde_json::from_str(&saved_json)?;
let title = &raw.product.title;
let variants = &raw.product.variants.edges;
```

All 17 configured Anker retailers share concrete types in `graphql/raw.rs`,
re-exported through their `offer_details/v2/raw.rs` compatibility paths and prelude. Every product
field is typed, including connections, images/media, variants, prices, quantity
rules and localization. Unknown fields fail rather than being silently dropped,
except the selling-plan fields of older captures, which are accepted and
ignored. Decimal amounts remain strings; other numeric scalars retain
JSON numbers. Diagnostic metadata and recorded continuation response pages stay
JSON evidence. Artifacts use schema 1 without context and schema 2 with context; v2 names the pipeline.
All supported storefronts now use the shared complete extracted model described below.

Each retailer's fixtures preserve its own captured product data. Run
`cargo test -p retailers_anker offer_details::v2` in `lib` for fixtures, or
`cargo test -p retailers_anker saved_graphql_corpus -- --ignored` for the external
`../data3` corpus. Nordic fixtures use the corresponding country context.
See [ADR 64](../../docs/adr/0064-brand-shared-graphql.md)
for ownership and [ADR 57](../../docs/adr/0057-offer-details-v2-raw-sources.md)
for the dump contract.

Regional US/EU/FR/NZ/PL/VN raw models also retain inventory, metafields and
expanded variant-reference selections from their saved full-catalog captures.
These optional source fields are local to those retailers; the initial-fill
query documents are unchanged; the shared extracted model retains these fields.

## Shared GraphQL implementation

`src/graphql` owns the brand’s queries, schema, raw types and integration engine. Storefront modules provide configuration, compatibility aliases and fixtures. HTML models remain storefront-specific; Anker extraction is shared as described below. See [ADR 64](../../docs/adr/0064-brand-shared-graphql.md).

## Shared complete extracted model

All 17 supported GraphQL Anker storefronts use `extracted::ExtractedOfferDetails`.
AU/CA compatibility exports and the other storefront extraction modules re-export
that same model. The former per-storefront converters are removed.

The layout groups product identifiers, taxonomy and content, with variants
inside the product and collections, images and media at the artifact root.
Product and variant image references use source IDs. Root images are unique by
ID, in first-observation order: product images, variant images, then featured
image. Identical repeated observations are skipped; conflicting extracted
fields for the same ID fail conversion with that ID in the error. Collection
and media images remain nested and are outside this root-image deduplication.

Money uses exact currency minor units and timestamps are parsed as UTC instants.
Acquisition time and market context are required. Raw parsing validates its source contract;
extraction checks supplied variant counts. Context is preserved as recorded.
GraphQL request/response pages remain in the raw artifact, not extracted output.

Run `make fmt` and `make ci`. Extracted tests cover field mapping, raw validation,
extracted serialization round trips, pricing, and unique image references.

## V2 conversion ownership

V2 raw models are defined in `src/graphql/raw.rs`. Extracted model definitions
are in `src/extracted/extracted.rs`; explicit per-type `TryFrom` implementations
are in `src/extracted/raw_to_extracted.rs`. These conversions use direct field
assignments and inline ordered connection traversal, without conversion helpers
or JSON round trips. The facade only parses raw, converts, and serializes.
Acquisition time is optional, optional extracted fields serialize as null, and
all extracted connections are vectors. GraphQL page evidence, explicit type-name
fields, and product price ranges are excluded. See ADR 66.

The extracted artifact owns root `collections`, `images`, and `media` vectors.
The product stores their ordered IDs (Anker collection IDs remain in taxonomy).
Moved image and media entries must have IDs; conversion returns an error if an
ID is absent or null. Anker uses ID references for featured
and variant images, preserves nested collection images, and requires IDs on all
root image records.

V2 extraction requires `acquired_at` and a resolved `context`. Acquisition time
must be a valid RFC 3339 instant. Explicit context is preserved. If raw context
is absent, country/language are recovered from recorded response
`extensions.context` and currency from variant prices and product price ranges,
including comparison prices. Missing, malformed or conflicting evidence fails.
Raw dumps are unchanged; see [ADR 67](../../docs/adr/0067-anker-context-recovery.md).

Fixed checkout charges use `CheckoutChargeValue::Money(money::Money)`, converted
through the same currency/precision checks as other prices. Extracted JSON uses
`{"__typename":"Money","amount":7999,"currency":"USD"}` for USD 79.99.
Percentage charges are unchanged. Raw GraphQL retains its `MoneyV2` source shape;
older extracted checkout values must be regenerated from raw artifacts.

Variant extraction rejects a compare-at price in a different currency, including
when the current price is zero. Errors identify the variant and both currencies.
Same-currency comparisons at or below the current price are omitted as before.

All 17 supported Anker storefronts use inventory main-table locales for initial
fills. Country is optional; currency is observed in responses. See ADR 69 in docs/adr.

Catalog and product GraphQL queries accept an optional country, including all
continuation queries. Calls without an explicit `MarketContext` omit the country
variable and use Shopify's default country while retaining the configured
language. Returned prices must use a consistent observed currency; they are not
forced to the configured country's currency. Explicit contexts still enforce
country, language and currency. The recorded responses retain Shopify's context
evidence. Initial fill now obtains requested contexts from the retailer inventory; see ADR 69.

Initial-fill collection loads `Retailer.supported_locales` through the existing
`StorefrontMarketsDiscovered` stage. It does not query advertised markets.
Advertised-only locales stay in the documentation. The local
`LocaleInitialFillIntegration` interface accepts optional countries and retains
response-derived country/language/currency in schema-2 artifacts. Historical
concrete-context integrations remain available as compatibility adapters.

The remote `Markets` GraphQL query has been removed. Initial fill reads
`Retailer.supported_locales`; the legacy trait method returns an error without
HTTP access. Saved localization fixtures remain historical evidence.
