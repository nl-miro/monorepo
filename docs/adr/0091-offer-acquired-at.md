# 91. One offer acquisition time, without versioned timestamp contracts

Status: Accepted. Supersedes ADR 26 and the export and creation-event version
clauses listed below.

## Context

The pipeline called the same instant `fetched_at`, `scraped_at` and, in the
Shopify models, `acquired_at`. The HTML column defaulted to the database clock
and a repeat download kept the first row's time. GraphQL extraction inserted the
command's time into the raw body only when the body lacked one. The three
contracts that carry the time, `OfferCreated`, the scrape-time update event and
the offer export, had grown ten, one and nine versions, each kept readable.

## Decision

`acquired_at` is the UTC instant when sourcing finishes successfully fetching an
offer from the retailer, truncated to microseconds. Both fetch handlers capture
it once through one helper. Rust, JSON, REST and SQL use `acquired_at`, and
GraphQL uses `acquiredAt`.

- The HTML download writes the time explicitly to `offer_details_raw.acquired_at`,
  which has no default. A repeat download replaces the object reference and the
  time together. The row, storage metadata and event carry the same instant.
- The GraphQL capture splices `"acquired_at":"<value>"` immediately after the
  raw root's opening brace and keeps every other byte. It first validates the
  whole document, and rejects a byte order mark, a non-object root and an
  existing `acquired_at`. Extraction requires the root time to equal the
  command's and passes the raw body on unchanged.
- Storage metadata, sourcing commands and events, exports and the outgoing
  import payload require the time. GraphQL import preparation treats storage
  metadata as authoritative. A structuralized variant with another time fails
  the whole batch, even a variant that would be skipped.
- Offers keeps the time optional on its import request, creation event, state
  and read models, because a manual REST import may omit it. Offers never
  synthesizes a time. Both projections keep the latest known time.

Versioning ends for the three timestamp-bearing contracts only:

- Offers persists creation as the literal tag `OfferCreated` and rejects
  `OfferCreated.v1` through `.v10`. The current creation fields are unchanged.
- `OfferScrapedAtUpdated.v1` becomes the unversioned `OfferAcquiredAtUpdated`
  in storage, publication and consumption.
- The offer export has one payload shape. Exports, export events and commands
  carry no `schema_version`, and `offer_export` is unique on
  `(offer_id, source_object_id)`.

Other event tags and Shopify raw artifact schema versions keep their contracts.
This ADR replaces these clauses:

| ADR | Replaced clause | Kept |
| --- | --- | --- |
| 19 | Schema version in the export key and event; version 1 as a fixed conversion contract | Immutable preparation, exact pending bytes and stable IDs |
| 24 | Explicit schema 2 selection, schema 1 for unversioned commands, the v1 shape without `images` | Append-only images and `OfferImageAdded.v1` |
| 25 | `OfferCreated.v2` tag and `.v1` reading | Ordered initial `images` in creation |
| 26 | All of it: `scraped_at`, export v3, `OfferCreated.v3`, `OfferScrapedAtUpdated.v1` and keeping v1/v2 shapes | Monotonic projected time |
| 28 | Export v4 and creation v4 while keeping v1–v3 | Required localization observation |
| 29 | Export v5 and creation v5 while keeping v1–v4 | Optional GTIN and SKU, present external identifiers |
| 34 | Export v6 with v1–v5 readers and `OfferCreated.v6` | Comparison pricing and `OfferPricingChanged.v1` |
| 35 | Export v7 and `OfferCreated.v7` | Availability and item condition semantics |
| 36 | Export v8 and `OfferCreated.v8` | Description content and `OfferContentChanged.v1` |
| 37 | Nothing. It added no version, and that still holds | Identity projection identifiers |
| 38 | `OfferCreated.v9` and separate v1–v8 creation decoding | Localized content lists and `OfferContentsChanged.v1` |
| 40 | Export v9 next to readable v1–v8, historical label regeneration, `OfferCreated.v10` | Region codes and `OfferLocalizationChanged.v2` |

## Consequences

Old rows, events and queued messages do not decode. The operator wipes offer
data in all three services before deployment, following the
[cutover runbook](../process/offer-acquired-at-cutover.md). The wipe also
removes manually imported offers. Old objects stay in object storage,
unreferenced. A future change to these contracts needs a new decision about
compatibility rather than another version.
