# Review: docs/retailer-architecture-gpt-6.md

- Date: 2026-10-14
- Reviewer: Claude Opus 5.5 (`claude-opus-5-5`), effort `high`
- Branch: `wip/processing-offers` at `67f5168`
- Scope: the whole document, read as the team that implemented the retailer
  storefronts

Each claim was checked against `lib/retailers_*`, `lib/retailer_data_extractor`,
`lib/retailer_sauce`, `services/retailer_sourcing` and the [retailer integration playbook](../retailer-integration.md).

## Summary

The outline is right: one library per brand, the sitemap hooks, and three
offer-details stages. But the document can't serve as a reference yet:

- It gets hook names and their meaning wrong, so code written from it doesn't
  compile or does the wrong thing.
- It leaves out how a call reaches a storefront (facade and registry), which is
  the first thing an implementer needs.
- It describes some ankerau or Anker-only behaviour as if every storefront did
  the same.

It reads like section 2 of the playbook with the specifics removed. It should
either become a short overview that links to the playbook, or be corrected and
filled in as described below.

## Factually wrong

### 1. `sitemap_gray_listed` doesn't exist

The hook is `sitemap_greylisted`, for example in
`lib/retailers_anker/src/ankerau/sitemap/v1/mod.rs`. The registry calls it
through `retailer_data_extractor::sitemap_greylisted_for`.

### 2. The greylist hook doesn't whitelist or blacklist

It is `fn(url: &str) -> bool`, and `true` means "download this child sitemap".
It is never called for root sitemaps, which are always downloaded
(`services/retailer_sourcing/src/sitemap_discovery/retrieve_sitemap/crawl.rs:131`).
For a retailer no crate knows, the service downloads every child
(`sitemap_discovery/mod.rs:46`).

### 3. The 1-second pause default belongs to the service

`DEFAULT_SITEMAP_PAUSE` in `sitemap_discovery/mod.rs` applies only when the
registry returns `None`. Every storefront returns its own value, and
`ugreencom`, `ugreeneu` and `ugreennaseu` use 5 seconds. "After every third
file" is correct (`DOWNLOADS_PER_PAUSE = 3` in `crawl.rs`).

### 4. `classify_link` takes three arguments

The signature is `classify_link(url, source, image_count)`. `source` is `"main"`
when the source sitemap is unknown.

### 5. The classification order is ankerau's, not a shared algorithm

"Scope, then source sitemap, then path, then unknown" is what ankerau does.
`ugreencom` has no separate region guard and matches a full prefix for each kind
(`lib/retailers_ugreen/src/ugreencom/sitemap/v1/mod.rs:30`), and most
storefronts never look at `source`. Present it as the recommended order, and say
the region guard comes first when several regions share a domain.

### 6. The path conventions don't match the helper

`retailer_sauce::shopify::shopify_from_location` matches `/pages/` and `/blogs/`
only. `/blog/` comes from `anker_from_location`
(`lib/retailers_anker/src/lib.rs:26`) and some storefront rules. Name the
helpers instead of restating their rules.

### 7. Extraction doesn't produce the `raw` types

`architecture()` returns an `html_extractor` `Architecture`, and extraction
produces untyped JSON keyed by the architecture's structure names. The `raw`
module is the strict serde schema, with `deny_unknown_fields` on every struct,
that `normalize` reads that JSON into. That boundary is where a key the survey
didn't see fails loudly. Merging the two steps loses the main design point.

### 8. Only Anker has a structuralizer

"Currently, this document contains the product variations" holds only for Anker
storefronts, and `ankerkr` returns an empty document. The facade's
`structuralize_for` calls only `retailers_anker`
(`lib/retailer_data_extractor/src/lib.rs:56`). Mi, Minisforum and Ugreen have
none.

## Missing for implementers

### 9. Registry and facade

How a call reaches a storefront:

1. `retailer_data_extractor::<hook>_for` tries each brand crate with `or_else`.
2. Each crate's `registry.rs` matches on `RetailerCode`, and `None` means "not my
   storefront".
3. The matching arm calls `<slug>::prelude::<hook>_v1`.

Without this, nobody can wire a storefront.

### 10. Library and storefront

"All Anker retailers belong to `retailers_anker`" blurs two levels. A library is
a brand. Each `RetailerCode` slug, such as `ankerau` or `ugreennasde`, is a
storefront folder inside it. The playbook says "storefront" throughout, and this
document should too.

### 11. Versioning is asserted but not described

"Versioning is part of the architecture from the start" needs the mechanism:

- the implementation lives in `v1/` modules;
- the prelude re-exports it with a `_v1` suffix;
- the registry decides which version to call.

It should also note a gap. Nothing records which version produced a stored
output, and `services/retailer_sourcing/migrations` has no version column. Once
a `v2` exists, nobody can tell which version wrote a stored normalized document.
Either record the version or list it as still to be decided.

### 12. Which pipeline step calls which hook

The document doesn't say which `retailer_sourcing` slice calls which hook, or
that the same hooks run offline through the `make process-*` tools. Link to the
table in section 2 of the playbook.

### 13. Errors and panics

- `v1` functions return `Result<_, String>`, and storefronts use typed
  `thiserror` errors inside.
- No panics on a finished storefront. The service catches them, but
  `process_offer_details` doesn't, so one panic stops the whole run.
- Stubs still call `todo!()`, for example `micom`'s `classify_link`.

### 14. Shared crates and value objects

Say what a storefront takes from each shared crate:

- `bodul_shared`: `RetailerCode`, `SitemapConfig`, `LinkKind`, `NormalizedData`
- `retailer_sauce`: `shopify_from_location`, `Availability`, `ItemCondition`,
  `StructuralizedData`
- `money`: `Money`

"Value objects" in the normalization section should name `Money`, `Availability`
and `ItemCondition`.

## Wording and consistency

### 15. Pick one spelling for catalog

The document says "Catalogue offers", the module is `catalog_offers`, and
`LinkKind` has `Catalog`. Use "catalog". Also say that every storefront already
has an empty `catalog_offers/v1` placeholder (46 of 46).

### 16. Comment out types, not mappings

The document says "comment out the corresponding mappings". ankerau comments out
the *types* in `normalized.rs` and keeps them as exclusion records with a
reason. Say so, so implementers don't delete them.

### 17. Give `Unknown` a target

The playbook says to aim for no `Unknown` URLs on real data and to explain any
that remain. Add that here.

### 18. Spell out the two rules implementers get wrong

In the structuralized field list, note that:

- each variation carries its own price;
- the URL is the canonical URL, or `canonical?variant=<id>` when there are
  several variants, and never the ld+json offer URL.

### 19. Link it, or merge it

Nothing links to this document. It overlaps with
`docs/retailer-architecture-qwen3.8-flash-next-ud-iq4_xs.md`, which repeats the
naming error from finding 1 (`sitemap_gray_list`). Keep one of the two and link
it from the playbook.
