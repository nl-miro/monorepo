# Anker AU catalog

Open the `bruno` workspace and select `ankerau_catalog`. These two requests
match the fixed Rust queries in `services/retailer_sourcing/src/platform/graphql/ankerau`.
Their GraphQL bodies are formatted for editing in Bruno.

- **catalog offers**: one page of up to 250 products, including availability,
  price ranges and up to 20 variants per product with IDs, prices and availability.
  The GraphQL variable `after` starts at `null`.
- **offer details**: details, collections, images, media, variants, selling-plan
  groups and price ranges for one product. Change `id` in the GraphQL variables;
  its default is `gid://shopify/Product/15395172516091`.

Offer-details page sizes are 25 collections, images and media; 20 variants and
selling-plan groups; and 5 components, grouped variants, quantity price breaks,
selling-plan allocations, store availabilities and selling plans. With these
limits, the query costs 1,525 points. Offer details sends Anker AU's public
Storefront token in `X-Shopify-Storefront-Access-Token` and succeeds with these
limits. Without the token, the endpoint rejects it with
`MAX_COMPLEXITY_EXCEEDED` (maximum 1,000).

The token comes from the public browser configuration on
[Anker AU's storefront](https://www.anker.com/au), in the `storeTokens.au` entry
of its JavaScript. It is a public Storefront token intended for browser use.
If Anker rotates it, update the header in `offer-details.bru` from that
configuration. Catalog offers remains tokenless. See Shopify's
[Storefront authentication documentation](https://shopify.dev/docs/api/storefront/2026-10#authentication).

Each request makes one HTTP call to Anker AU's public Storefront API in AU/EN
context. Connection `pageInfo` indicates whether more items exist; these requests
do not follow pagination automatically.

Validate with Bruno CLI 4.2.1:

```sh
cd bruno/ankerau-catalog
bru run
```

Both request tests check HTTP success, GraphQL errors and result presence.
Catalog offers also checks the tokenless 1,000-point query-cost limit.
