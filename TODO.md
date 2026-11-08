
GOAL: Switching Shopify retailers to use Shopify API

- [x] Move the three GraphQL documents into an `ankerau` subfolder and update all client references.
- [x] Pretty-print saved GraphQL JSON responses in all three clients and format existing artifacts.
- [x] Sync both Bruno requests with all three Rust clients and overwrite the six saved request/response pairs.
- [x] Use Anker AU's public Storefront token for Bruno's offer details request and verify the response.
- [x] Fix the Bruno request type so catalog offers and offer details display their GraphQL bodies.
- [x] Replace the Anker AU catalog collection with only `catalog offers` and `offer details`, removing the duplicate offers collection and obsolete pagination files.
- [x] Increase offer-details connection limits across all clients and matching Bruno requests.
  Live validation rejects the requested query with MAX_COMPLEXITY_EXCEEDED:
  cost 1,525 exceeds the public endpoint's 1,000 limit in all three clients.
- [x] Increase catalog offers to 20 variants per product across Bruno and all three Rust clients.
- [x] Pretty-print `schema.graphql` and `offer_details.graphql` without changing their meaning.
- [x] Add single-call `catalog offers` and `offer details` Bruno requests matching the Rust clients.
- [x] Pretty-print `catalog_offers.graphql` and keep its Bruno parity check insensitive to formatting.
- [x] Remove the old GraphQL `Field`/`Dump` downloader and its obsolete entry points, keeping the two fixed operations in all three clients.
  Move the fixed operations into the client modules; remove the old download
  APIs, CLI, comparison tests and schema-discovery fixtures. Supersede ADR 46.
- [x] Implement Bruno's catalog offers and offer details operations in the custom, graphql_client and Cynic clients, with separate live-test request/response evidence for each implementation and operation.
  Execute each query once per implementation. Save exactly one request.json and
  response.json per operation in stable implementation/operation folders.
  No retries, pagination calls, run IDs, summaries or assembled results.
- [x] Increase `08 availability and prices` to the first 500 products, retaining complete variant pagination.
  Fetch up to two catalogue pages of 250 products and merge their variants.
- [x] Add an Anker AU Bruno request named `single` that fetches all existing field groups for a given product ID.
  Combine the seven field selections and drain connections into one response.
- [x] Add a catalogue request for the first 50 products with product and variant IDs, prices and availability.
  Keep the catalogue limited to 50 products and drain each product's variants.

- [x] Skip `data.product.compareAtPriceRange` in the Anker AU GraphQL details requests in Bruno.
- [x] Remove `data.product.priceRange` from the Anker AU GraphQL details requests in Bruno.
- [x] Create a new Anker AU GraphQL request in Bruno that fetches only fields removed from the existing requests, including `data.product.compareAtPriceRange` and `data.product.priceRange`.
- [x] Check whether fetching `data.product.media` also covers the images we need, or whether separate `data.product.images` requests are still necessary for Anker AU.
  Result (2026-11-08): the fully paginated Bruno responses from the earlier live
  run contain the same 54 images across the five products (6, 24, 12, 7, 5).
  Filtering `media.edges[].node` to `MediaImage` and reading `.image` preserves
  their order, URLs, alt text, dimensions, thumbhashes and image type. IDs differ:
  `images` returns `ProductImage` IDs; `MediaImage.image` returns `ImageSource`
  IDs. Media alone therefore covers the image content needed for our URL-based
  offer galleries in this sample. Separate images requests are needed to retain
  the original ProductImage IDs and exhaustive field-path coverage, so the
  review collection keeps them. Omitting them from a content-only fetch would
  save eight requests with the current page size of ten.
  Use the actual `MediaImage.image`, not video/model preview images, and paginate
  media to completion. The five sampled products contain only image media;
  this comparison does not prove equivalence for every product or ordering mode.
  References: [Product media](https://shopify.dev/docs/api/storefront/2026-10/objects/Product#field-Product.fields.media)
  and [MediaImage.image](https://shopify.dev/docs/api/storefront/2026-10/objects/MediaImage#field-MediaImage.fields.image).

- [~] Convert the Anker AU Bruno collection to catalogue pagination with five products per page and complete per-product connection pagination.



Status follows the [TODO workflow](AGENTS.md#todo-workflow):

| Marker | Phase                      |
| ------ | -------------------------- |
| `[ ]`  | Pending                    |
| `[.]`  | Planning in progress       |
| `[>]`  | Planning complete          |
| `[~]`  | Implementation in progress |
| `[x]`  | Implemented                |

Flow: `[ ]` → `[.]` → `[>]` → `[~]` → `[x]`.

NOTES:
- We are aware that `For now` is going against ADR-16, but this is shortterm
- depending on local mulac will be needed in this phase and possibly next ones
- `customer-facing` will take given events and persist them into `offers` table


BACKLOG: leave this alone
- prepare offer details versioning 
- Determine item condifition from title (refusrbished ...)
- Refactor how offer duplication is handled: use gtin or sku per retailer?
- Refactor commanding by introducing priority column, first command starts with priority 100 and every after it does -1 on current priority. If we reach 10, we stop with -1 and use last one. That way we fast track first batch to finish point. When reserving commands, it should also order by priority asc
- Drop descripton column on `customer_facing`'s 'offer' table
