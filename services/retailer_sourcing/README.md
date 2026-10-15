# retailer_sourcing

Finds what retailers sell. It crawls each retailer's sitemaps, sorts the page
URLs into products, catalogs and other content, records every product URL as an
offer, downloads the offer pages, and reads the offer data out of them: it
extracts each page into JSON, normalizes it and structuralizes it.

The code is organised as feature slices
([ADR 4](../../docs/adr/0004-feature-slices-for-retailer-sourcing.md)). Each
slice handles one command and returns events; a reaction in the next slice turns
an event into the next command. Commands and events are stored in Postgres by
the [Mulac](https://github.com/nulllabsdev/mulac) kernel.

## Workflow

```
POST /api/sitemap-retrievals
  └─▶ RetrieveSitemap ──▶ retrieve_sitemap handler ──▶ SitemapRetrieved
        └─▶ process_sitemap reaction ──▶ ProcessSitemap ──▶ process_sitemap handler ──▶ SitemapProcessed
              └─▶ consolidate_sitemap reaction ──▶ ConsolidateSitemap ──▶ consolidate_sitemap handler ──▶ SitemapConsolidated
                    └─▶ discover_offers reaction ──▶ DiscoverOffers ──▶ discover_offers handler ──▶ OfferDiscovered × N
                          └─▶ download_offer_page reaction ──▶ DownloadOfferPage × N ──▶ download_offer_page handler
                                ──▶ OfferPageWasDownloaded | no event (download failed)
                                  └─▶ extract_offer_page reaction ──▶ ExtractOfferPage ──▶ extract_offer_page handler ──▶ OfferPageWasExtracted
                                        └─▶ normalize_offer reaction ──▶ NormalizeOffer ──▶ normalize_offer handler ──▶ OfferWasNormalized
                                              └─▶ structuralize_offer reaction ──▶ StructuralizeOffer ──▶ structuralize_offer handler ──▶ OfferWasStructuralized
```

| Step | Command              | Handler does                                                                                                                                                                                                    | Event                                                    | Reaction that follows                                                 |
| ---- | -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- | --------------------------------------------------------------------- |
| 1    | `RetrieveSitemap`    | Fetches the retailer's root sitemaps and every child sitemap and stores the files in `sitemaps_raw`. A file that can't be fetched or parsed is skipped with a warning. Fails without a sitemap config. | `SitemapRetrieved`                                       | `process_sitemap`: queue `ProcessSitemap`                             |
| 2    | `ProcessSitemap`     | Parses the stored files into one sitemap tree, stores it in `sitemaps_processed`.                                                                                                                               | `SitemapProcessed`                                       | `consolidate_sitemap`: queue `ConsolidateSitemap`                     |
| 3    | `ConsolidateSitemap` | Sorts every URL of the tree into product, catalog, content, not interested and unknown, using the retailer's link rules; stores the groups in `sitemaps_consolidated`.                                          | `SitemapConsolidated`                                    | `discover_offers`: queue `DiscoverOffers`, unless it found 0 products |
| 4    | `DiscoverOffers`     | Reads the product URLs and gives each its own `offer_id`. Writes nothing.                                                                                                                                       | `OfferDiscovered`, one per URL                           | `download_offer_page`: queue one `DownloadOfferPage` per offer        |
| 5    | `DownloadOfferPage`  | Downloads the page and stores it in `offer_details_raw` under the `offer_id`, with its retailer code. A failed download is logged, and the command still completes.                                                    | `OfferPageWasDownloaded`; none when the download failed  | `extract_offer_page`: queue `ExtractOfferPage`                        |
| 6    | `ExtractOfferPage`   | Extracts the offer's page from `offer_details_raw` into JSON with the retailer's architecture and stores it in `offer_details_extracted`.                                                                                     | `OfferPageWasExtracted`                                  | `normalize_offer`: queue `NormalizeOffer`                             |
| 7    | `NormalizeOffer`     | Normalizes the extracted JSON with the retailer's normalizer and stores it in `offer_details_normalized`. When the retailer produces no document for the page, stores nothing.                                         | `OfferWasNormalized`; none when there is no document     | `structuralize_offer`: queue `StructuralizeOffer`                     |
| 8    | `StructuralizeOffer` | Structuralizes the normalized document with the retailer's structuralizer and stores it in `offer_details_structuralized`. When the retailer produces no document, stores nothing.                                     | `OfferWasStructuralized`; none when there is no document | none yet                                                              |

Steps 6 to 8 fail the command when the input row is missing, when the retailer
has no such step, or when the step fails, including a retailer whose step is
still a `todo!()` stub; the bus retries a failed command on a later run. Only
`ankerau` has every step implemented so far. Each of their tables holds one row
per offer, replaced when the step runs again.

Each table keeps its payload, the fetched body or the JSON document, in a
`<table>_data` table keyed by the row's `id`, such as `raw_offers_data`. Rows
can be read and listed without loading their payloads, and a row and its payload
are written in one transaction.

Every row a crawl writes in steps 1 to 3 carries its `retrieval_id`. Each step
logs its outcome (see [Logging](#logging)); `command_entries` and
`event_entries` show what is queued, done or failed.

## Code layout

```
src/
  main.rs                    routes, startup; registers each context
  platform/                  plumbing that knows nothing about the domain
    bus.rs                   command and event bus over Mulac
    bus_api.rs               POST /api/commands/run and /api/events/run
    db.rs                    Postgres pool and embedded migrations
    http.rs                  blocking HTTP client
  sitemap_discovery/         context: sitemaps → sorted URLs
    retrieve_sitemap/        step 1, plus POST /api/sitemap-retrievals
    process_sitemap/         step 2
    consolidate_sitemap/     step 3
  offer_discovery/           context: product URLs → offer data
    discover_offers/         step 4
    download_offer_page/     step 5
    extract_offer_page/      step 6
    normalize_offer/         step 7
    structuralize_offer/     step 8
  development/               context: tooling for working on the pipeline; no slices yet
  monitoring/                context: the pipeline's health; no slices yet
  reporting/                 context: reports over sourced data; no slices yet
migrations/                  Mulac's tables, then each context's tables
```

A slice holds `command.rs`, `handler.rs`, `event.rs`, `entity.rs` (domain
values), `record.rs` (PostgreSQL representations and conversions), `store.rs`
(its own queries), and `reacts_to.rs` when a reaction starts it. Each context and
slice exposes only its `prelude` module, with `register()`, any `routes()`, and
the events others may use. Contexts talk to each other only through events.

## Running it

The service needs Postgres for the workflow; without `DATABASE_URL` it serves
HTTP only and the routes below return `503`. On startup it applies pending
migrations.

```sh
make .env          # once: DATABASE_URL for the retailer_sourcing-db container
make migrate       # optional; startup also migrates
PORT=3002 make serve
```

`make migrate-redo` reverts and reapplies every migration, which deletes the
data.

When the service runs in the workspace's `compose-apps.yaml` container, rebuild
and restart it after code changes, from the workspace folder:

```sh
make services-restart
```

Events are delivered to the reactions registered in the running binary. An event
processed by an older build that had no reaction for it is marked done and isn't
delivered again.

## Driving the workflow

Nothing processes the queues in the background (the polling workers are
commented out in `Bus::start`). Queue retrievals, then run commands and events
in turns:

| Route                          | Body                          | Does                                                                      |
| ------------------------------ | ----------------------------- | ------------------------------------------------------------------------- |
| `POST /api/sitemap-retrievals` | `{"retailers": ["ugreenca"]}` | Queues a `RetrieveSitemap` per retailer, returns each `retrieval_id`      |
| `POST /api/commands/run`       | `{"limit": 10}`               | Runs up to `limit` queued commands and waits for them                     |
| `POST /api/events/run`         | `{"limit": 10}`               | Runs up to `limit` queued events; their reactions queue the next commands |

One retailer from start to finish takes alternating runs:

1. `sitemap-retrievals` with the retailer.
2. `commands/run`: retrieves the sitemaps (can take minutes; keep the limit small).
3. `events/run`, then `commands/run`: processes the sitemaps.
4. `events/run`, then `commands/run`: consolidates them.
5. `events/run`, then `commands/run`: discovers the offers.
6. `events/run`, then `commands/run` until it reports `"processed": 0`: downloads the offer pages.
7. `events/run`, then `commands/run`, three more times: extracts, normalizes and structuralizes the offers.

Run responses look like `{"processed": 3, "errors": []}`. When something fails,
`processed` is `null` and `errors` lists the failures; failed commands are
retried on a later run.

The Bruno collection in `../../bruno/retailer_sourcing` has a request for each
route.

## Exporting offer pages

`make export-offer-sources` writes every page in `offer_details_raw` to
`<DATA_DIR>/<retailer>/offer_details/sources/<url path>.html`, the layout of the
per-retailer data repos. `DATA_DIR` defaults to `../../../data`, the `data`
folder next to the monorepo. A page's file is named after its decoded URL path
(`au/products/a1256` becomes `au-products-a1256.html`) and is overwritten on the
next export.

`make export-sitemaps` writes each retailer's most recently processed sitemap
crawl to `<DATA_DIR>/<retailer>/sitemap`, one folder per stage, each named for
the crawl's retrieval id:

- `sources/<retrieval id>/<url path>.xml`: every sitemap file as fetched.
  Names follow the offer pages' rule, plus the query with `&` replaced by `_`,
  since Shopify pages its sitemaps by query (`sitemap_products_1.xml?from=1&to=2` becomes
  `sitemap_products_1_from=1_to=2.xml`).
- `processed/<retrieval id>.json`: the files parsed into one sitemap tree.
- `consolidated/<retrieval id>.json`: the tree's URLs grouped by kind of page,
  once the crawl has been consolidated.

Exporting the same crawl again overwrites its files. A newer crawl gets its own
folder and files, so earlier exports stay alongside it.

## Logging

Logs go to stderr and to daily files under `logs/` (set `LOG_DIR` to change the
folder; it's git-ignored):

| File                               | Contains                                                                                                                       |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `retailer_sourcing.log.YYYY-MM-DD` | The same lines as stderr: each step's outcome, skipped sitemap files, failed downloads and commands, one line per HTTP request |
| `timing.log.YYYY-MM-DD`            | How long each fetch, command and reaction took, and whether it succeeded                                                       |

`RUST_LOG` sets the level, for example `RUST_LOG=debug`; the default is `info`.
In the compose container the files are written inside the container, so mount a
volume on the log folder to keep them.

## Tests

`make ci` runs formatting, clippy and the tests. Handler tests use in-memory
fakes, so they need no database or network. `make test-postgres` checks store
queries and transactions in isolated schemas; see the database test setup below.

## Domain entities and PostgreSQL records

All three sitemap discovery slices and all five offer discovery slices follow
[ADR 7](../../docs/adr/0007-domain-entities-and-postgres-records.md).
Each slice's `entity.rs` contains domain types; `record.rs` contains private
Diesel types ending in `PgRecord` and their explicit conversions. Conversions
that parse stored JSON or check integer bounds return errors. `PgStore` takes
entities when writing and IDs when reading, and returns entities when loading.
Only the store handles PostgreSQL records. Queries remain owned by the slice;
entities and records are not imported from another slice.

- Retrieval passes a batch of `RetrievedSitemapDocument` entities, each carrying
  its ID, retrieval ID, timestamp, and the sitemap library's raw document.
- Processing reads the library's `RawSitemapDocument` domain values and writes a
  `ProcessedSitemap`. Its URL count is a `usize`; conversion to the stored `i32`
  happens in `record.rs`. Negative stored raw sizes are rejected on reads.
- Offer discovery reads a `ConsolidatedSitemap` entity containing product URLs.
- Downloading writes a `RawOffer` entity containing the offer ID, retailer, URL,
  and downloaded body.
- Extraction reads `RawOffer` and writes `ExtractedOffer`; normalization reads
  `ExtractedOffer` and writes `NormalizedOffer`; structuralization reads
  `NormalizedOffer` and writes `StructuralizedOffer`. These are slice-local
  projections identified by the domain offer ID. JSON documents remain typed
  JSON values until the handler calls a retailer step that expects text.

The database's offer row IDs are persistence identities: upserts read back the
existing row ID and use it for the payload record. Each main/payload write stays
in one transaction. This separation changes no database columns or event shapes
for retrieval, processing, or offer discovery.

`ConsolidatedSitemap` owns five count fields, calculated from its grouped links
at construction and exposed through read-only accessors. Its PostgreSQL record
stores them in `sitemaps_consolidated.stats` as a JSONB object:

```json
{"product_count":12,"catalog_count":3,"content_count":7,"not_interested_count":2,"unknown_count":0}
```

The same record stores source references and storage information in two more
non-null JSONB columns, as recorded in [ADR 8](../../docs/adr/0008-consolidated-sitemap-jsonb-references-and-storage.md):

```json
{
  "references": {
    "processed_sitemap_id": "019a1234-5678-7000-8000-000000000001",
    "retrieval_id": "019a1234-5678-7000-8000-000000000002",
    "retailer_code": "ugreenca"
  },
  "storage": {"content_size": 123}
}
```

These four fields exist only inside JSONB, with no standalone or generated
columns. A unique expression index on the processed UUID in `references` keeps
one consolidated record per processed sitemap. There is no foreign key to the
processed row: deleting it now leaves the consolidated record and payload
intact. Deleting the consolidated record still cascades to its payload table.
Sitemap exports join through the JSONB reference and keep the same output format.

Grouped links remain in the payload table, and entity and event fields retain
their existing names and types. Counts and content size remain limited to the
previous `i32` storage range for downgrade support. In SQL, quote the reserved column name as `"references"`.

### Applying the migrations

Migration `2026-10-15-000009_rename_pipeline_tables` renames the seven pipeline
tables to `offer_details_raw`, `offer_details_extracted`,
`offer_details_normalized`, `offer_details_structuralized`, `sitemaps_raw`,
`sitemaps_processed`, and `sitemaps_consolidated`. It preserves rows, IDs,
indexes, and foreign keys; companion `_data` tables retain their existing names.
Stop old service instances before applying it with `make migrate` or starting
the new binary. To roll back the renames, stop the new service and run
`diesel migration revert` while migration 9 is latest, then start the old binary.
Revert migration 9 before following the older rollback instructions below.

Stop existing service instances before running `make migrate` or starting the
new binary (which applies pending migrations). Migration
`2026-10-15-000007_consolidated_sitemap_stats` backfills stats from the count
columns. Migration `2026-10-15-000008_consolidated_sitemap_references_storage`
backfills the references and storage objects before removing the four columns.
Both preserve existing values, IDs, timestamps, and payloads. The second
migration intentionally removes the processed-sitemap foreign key. Old binaries
cannot run against the new schema.

For rollback to the previous stats-only layout, stop the new service and, while
migration 8 is the latest applied migration, run `diesel migration revert` from
this directory with `DATABASE_URL` configured. Then start the previous binary.
Reverting migration 7 as well restores the original count columns. Do not use
`make migrate-redo` for rollback: that target reverts all migrations and deletes
stored data. Missing, malformed, or overflowing values cause rollback to fail
transactionally. Dangling processed references must be resolved before reverting
migration 8, because rollback restores the original foreign key.

### PostgreSQL tests

```sh
make fmt
make ci
TEST_DATABASE_URL=postgres://retailer_sourcing:retailer_sourcing@172.28.100.10:5432/retailer_sourcing_test make test-postgres
```

The test database must already exist, and its role must be able to create/drop
schemas. Each test creates a unique schema and removes it afterward, including
on assertion failure; it never resets existing tables. The target fails when
configuration or connectivity is missing. Ordinary `make test` leaves these
PostgreSQL tests ignored. Tests cover entity-facing reads and writes in every
sitemap and offer slice, missing inputs, conversion failures, retrieval ordering,
Unicode byte sizes, batch rollback, stable upsert IDs, failed insert/update
rollback, fresh
migrations, populated upgrade/downgrade/re-upgrade, failed downgrades, the export
query, reference uniqueness, and deletion behavior.
