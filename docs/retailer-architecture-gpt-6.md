# Retailer Architecture

## Retailer libraries

Retailer data is grouped into libraries. For example, all Anker retailers belong
to the `retailers_anker` library. Each library contains everything related to
its retailers' sitemaps, offer details, and catalog offers.

Versioning is part of the architecture from the start.

Of the 46 storefronts, 25 are stubs that panic when called rather than returning
a "not mine" result. The interfaces described below do not imply that every
storefront has a working implementation.

Catalog offers will be used and defined at a later stage.

## Sitemap discovery

For many retailers, sitemap discovery is the first step in retailer sourcing.
Retailer libraries expose the following functions and configuration.

### `sitemap_config`

Returns a sitemap configuration structure containing all sitemap URLs or root
URLs needed to begin discovery.

### `sitemap_greylisted`

Returns `true` when a child sitemap should be downloaded. Root sitemaps are
always downloaded.

### `sitemap_pause`

Configures a retailer's pause duration to avoid overwhelming it with sitemap
requests. Every storefront sets its own pause. The service's one-second default
applies only to retailers that no library recognizes. The Ugreen storefronts
`ugreencom`, `ugreeneu`, and `ugreennaseu` use five seconds. The pause occurs
after every third sitemap file downloaded.

### `classify_link`

Classifies a URL extracted from a sitemap XML file. It also takes the source
sitemap and `image_count` as context. When the source sitemap is unknown, it is
passed as `"main"`. The possible results are:

- Not interested: the URL is outside the retailer's sourcing scope.
- Product: the URL points to a product.
- Catalog: the URL points to a catalog or collection.
- Content: the URL points to other content, such as a page or blog.
- Unknown: no classification rule matches.

Classification rules and their order are retailer-specific. For example,
`ankerau` applies the following order; `ugreencom` works differently:

1. Check whether the URL is in scope. For example, reject a URL that does not
   start with a required prefix as not interested.
2. Check the source sitemap. A retailer can use the sitemap the URL came from to
   decide whether it is of interest and classify it as product, catalog, or
   content.
3. If still unclassified, check the URL itself. Path rules identify `/products/`
   and `/product/` paths as products, `/collections/` and `/collection/` paths as
   catalogs, and `pages`, `blogs`, or `blog` paths as content.
4. Return unknown if no rule matches.

## Offer details

The offer details architecture has three processing stages: extraction,
normalization, and structuralization.

### Extraction

The storefront's `architecture()` function defines the extraction configuration
for its offer details pages, which are usually HTML. The `html_extractor` crate
uses this configuration to extract untyped JSON.

### Normalization

The `normalize` process first reads the extracted JSON into the strict schema
defined in the `raw` module and its submodules. Unexpected keys fail loudly at
this boundary. It then produces a normalized document, creating value objects
wherever possible.

In the early stages, all raw data is mapped into the normalized document. As we
learn which data is unnecessary, we will comment out the corresponding mappings.

### Structuralization

The `structuralize` process, currently implemented only for Anker, takes the
normalized document and produces a structuralized document. This document
contains the product variations from each offer details page, with the following
data for each variation:

- Identifiers
- Title
- URL
- Price
- Images

The data included in the structuralized document will expand later.
