# Anker AU catalogue

This collection pages through the Anker AU Shopify Storefront catalogue in
ascending product ID order: **50 products per details page**, and **five per page
for all other field groups**. It uses Storefront
2026-10 with AU/EN context and no access token. Open the `bruno` workspace and
select `ankerau_catalog`.

The seven field-group requests cover details, collections, images, media, variants,
sellingPlanGroups and removedFields. Each request independently traverses the
catalogue before the runner proceeds to the next field group. The last catalogue
page can contain fewer than five products. Results are in
`data.products.nodes`, with catalogue pagination in `data.products.pageInfo`.

Connection requests initially fetch ten items per product, or five for variants
to keep the combined query within the 1,000-point tokenless limit. If products
have more items, the shared collection scripts batch the unfinished products
using aliases (`data.p0`, `data.p1`, etc.), each with its own cursor. These calls
contain up to five products. Match responses by product `id`; alias positions
can change as products finish. Only after all five products' connections finish
does the request advance to the next catalogue page.

Use the collection runner for automatic pagination. Sending manually sends one
page; runtime state allows subsequent sends to continue. State clears on a
successful finish or response-validation error. After interrupting a run or a
transport failure, clear runtime variables to restart from the beginning.
The response pane shows the latest page; use the CLI JSON report to retain all
pages. The scripts do not merge responses into a single payload.

`removedFields` contains `compareAtPriceRange` and `priceRange`, plus product
identity. Details omit those price ranges. `coverage.json` records the field
partition and page sizes. Only product IDs repeat across top-level field groups.
Images remain separate to preserve ProductImage IDs; image content is also
available through MediaImage.image with different ImageSource IDs.

All original selections are retained. Metafields, totalInventory,
quantityAvailable and shopPayInstallmentsPricing remain excluded where the
original requests require extra scopes. In the seven field-group requests,
deeper connections such as variant
components and selling plans retain their original finite selections: if their
`hasNextPage` becomes true, the runner stops with an explicit incomplete-export
error. It does not silently discard those pages. Missing products, repeated
cursors, duplicate catalogue IDs, GraphQL errors and excessive costs also stop
the run. Catalogue changes during a run can affect results; these queries do not
provide a transactional snapshot across field groups.

## Single product and first-500 prices

`single` combines all seven field groups for one product: details, collections,
images, media, variants, selling-plan groups and both price ranges. Set
`productId` in the request's **Vars** tab to a numeric Shopify product ID or a
full `gid://shopify/Product/...` ID. An environment `productId` overrides the
request default. The default is the sample product `15395172516091`.

`08 availability and prices` fetches the first **500 products in ascending ID
order**, using up to two catalogue pages of 250 products (or fewer products if
the catalogue is smaller). Each product includes `id`,
`availableForSale`, `priceRange` (minimum/maximum variant prices with currency),
and `variants.edges[].node` with each variant's `id`, `price` (amount/currency)
and `availableForSale`. Product prices are ranges because variants can differ.

Both requests automatically finish all their connection pages, including nested
connections in `single`, and merge them into the response pane. This works with
**Send** as well as the runner. `single` returns `data.product`; prices returns
`data.products.nodes`. The prices request stops after those 500 products even
when `data.products.pageInfo.hasNextPage` is true, but still retrieves every
variant for them. `catalogPageLimit` does not affect either request.

Each additional HTTP call is checked against the 1,000-point cost limit. The
merged response includes `extensions.pagination` with the HTTP request count,
maximum requested query cost and connection-completion status. Connection
`pageInfo` reflects the final page; edge cursors are retained. A failed page
fails the request instead of marking a partial result complete. These requests
use the same public field scope as the existing queries; they do not add fields
that require extra access scopes. As with catalogue runs, edits to products
during pagination can affect results.

Run either request independently from the collection directory:

```sh
bru run single.bru --env-var productId=15395172516091
bru run 08-availability-prices.bru --reporter-json /tmp/ankerau-prices.json
```

The runner also executes these two requests after the seven field groups when
running the whole collection.

## One page per field group

Select the `first-five` environment in Bruno, then run the collection. This
sets `catalogPageLimit=1`: details fetches the first 50 products; other groups
fetch the first five products,
finishes their connection pages, and stops before the next catalogue page.

```sh
bru run --env first-five --reporter-json /tmp/ankerau-first-five-bruno.json
```

## Validation

Requires Node.js and Bruno CLI 4.2.1. Shared scripts use built-in Bruno APIs;
the offline pagination tests use Node's built-in test runner with no packages.

```sh
node --test bruno/ankerau-catalog/tests/pagination.cjs
npm install -g @usebruno/cli@4.2.1
cd bruno/ankerau-catalog
bru run --env-var catalogPageLimit=2 --reporter-json /tmp/ankerau-catalog-sample.json
```

`catalogPageLimit` limits catalogue pages **per field group**, while still
finishing every connection page for the sampled products. A limited run logs
that it is partial if more products exist. The default is `0` (entire catalogue):

```sh
bru run --reporter-json /tmp/ankerau-catalog.json
```

The collection tests validate successful responses, requested query cost at most
1,000, page shapes and pagination progress. Inspect
`response.data.extensions.cost.requestedQueryCost` in the CLI report. Each group
also leaves `ankerCatalog.<group>.result` runtime metadata with product/page
counts and a `complete` flag.

The validated first-five run is saved in `artifacts/first-five/`: separate
`request.json` and `response.json` files for each successful call, plus a manifest
with product IDs and reported query costs.
