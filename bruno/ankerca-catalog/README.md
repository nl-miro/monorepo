# Anker Canada catalog

Open the `bruno` workspace and select `ankerca_catalog`. This collection has
only **catalog offers** and **offer details**, matching the Rust documents in
`services/retailer_sourcing/src/platform/graphql/ankerca/`.

Both requests use `https://anker-canada.myshopify.com/api/2026-10/graphql.json`
and CA/EN context with CAD prices. Catalog offers is tokenless and requests
250 products, 20 variants per product, IDs, availability and prices. Its `after`
variable starts at `null`.

Offer details sends the public browser Storefront token in
`X-Shopify-Storefront-Access-Token`. It requests 25 collections/images/media,
20 variants/selling-plan groups, and 5 items in each selected nested connection,
matching AU. Its test does not impose the tokenless 1,000-point cost limit.

The endpoint and token come from `storeDomains.ca` and `storeTokens.ca` in
[Anker's public storefront JavaScript](https://one.anker.com/_next/static/chunks/1b2878kmn7hql.js),
fetched on 2026-11-08. If the token rotates, inspect the scripts loaded by
[Anker Canada](https://www.anker.com/ca) and update both this header and
`Storefront::token` in Rust. See
[Shopify's public token documentation](https://shopify.dev/docs/api/storefront/2026-10#authentication).

**Live validation is pending:** the current environment cannot resolve the
storefront host. The details GID `gid://shopify/Product/8302268514493` comes from
existing Canada fixtures and is provisional. With network access, run catalog
offers, choose the first available product in its ID-sorted results, and update
both the details variables and the Canadian live test's GID. Then run:

```sh
cd bruno/ankerca-catalog
bru run
```

Bruno CLI 4.2.1 parses both requests. Tests check HTTP success, absence of
GraphQL errors and result presence. Catalog offers also checks the tokenless
query cost. Each request sends once; pagination is exposed through `pageInfo`
and is not followed automatically. Production integration and French requests
are outside this collection.
