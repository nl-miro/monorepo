# Retailer architecture

## Retailer libraries

Retailer data is grouped into libraries, one library per retailer family. All
Anker retailers, for example, live in a library named `retailers_anker`.

A library holds everything about the retailers it contains: their sitemaps,
offer details and catalog offers. When you need to know how a retailer's data is
discovered, or how its offers are described, the library is where you look.

Catalog offers are part of this architecture but are not described here yet:
they will be used and defined at a later moment.

## Versioning

Versioning happens per stage, not per crate: the Cargo version of every library
is 0.1.0 and carries no meaning here. Each stage lives in its own versioned
module — `sitemap/v1`, `offer_details/v1` and `catalog_offers/v1` — and the
library's prelude re-exports each function with a version suffix, such as
`classify_link_v1`. A v2 can therefore sit next to a v1 in the same library, and
the registry decides which version to call for each retailer. Changing how a
stage behaves is an explicit, identifiable step rather than an untracked edit.

## Sitemap discovery

For many retailers, sitemap discovery is the first step of retailer sourcing.
Each retailer defines a few functions for it.

### `sitemap_config`

Returns a configuration structure with all the sitemap URLs, or the root URLs to
start from, for the retailer. A root we do not want is simply left out of this
configuration.

### `sitemap_greylisted`

Given the URL of a child sitemap we are about to download, returns whether it
should be fetched. It only applies to child sitemaps: dropping a root is done by
removing it from `sitemap_config`. Of the 46 stores, 29 reject any child sitemap
whose file name contains `metaobject`, the Shopify sitemaps that list
metaobjects rather than products, and the other 17 greylist nothing and always
return true.

### `sitemap_pause`

To avoid hogging a particular retailer, each retailer defines a pause duration.
The default is one second, applied after every third sitemap file downloaded.

### `classify_link`

Takes a URL extracted from a sitemap XML file, together with the sitemap file
that listed it and the URL's image count, and decides whether we are interested
in it and, if we are, what kind of page it is. The result is one of:

- Not interested: for example, the URL does not belong to the retailer's
  domain
- Product: a product page
- Catalog: a catalog page
- Content: a content page
- Unknown: none of the above matched, so it is up to the caller to decide

Most rules match on the URL alone, such as Shopify's `/products/`,
`/collections/`, `/pages/` and `/blogs/` paths, but a retailer may also classify
by the sitemap file that listed the URL: Anker does this for most of its pages.

## Offer details

### `raw`

Once a product page is found, we extract its offer details. The extractor
returns JSON, and `raw` is the strict typed schema we deserialize that JSON
into. It is defined in the `raw` module, with submodules when needed.

Every struct in the schema denies unknown fields, so a key the survey never saw
fails loudly instead of being silently dropped.

### `normalize`

Turns the raw schema into value objects, for the data that can be represented
that way. The raw schema covers all of the extracted data; fields we choose not
to normalize stay in it, commented out with a reason, so the gap between what
extraction gives us and what we use stays visible.

### `structuralize`

Turns the normalized data into the structure we want to store, keeping only what
we need: for now, the product variations with their identifiers, title, URL,
price and images. Anker is the only retailer with a structuralizer so far, and
this is what it keeps; the set will expand as other retailers are added.

Each variation carries several identifiers: its GTIN, its SKU and, for Shopify
retailers, the Shopify product and variant GIDs and the handle. Each one is a
newtype that validates its own format.
