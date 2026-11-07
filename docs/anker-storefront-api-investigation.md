# Anker storefront product API investigation

Investigation date: 2026-11-07. Covers all 14 Anker retailer codes currently
registered in `lib/retailers_anker`, including Australia.

All 13 Shopify-backed retailers returned product data through Shopify's
Storefront GraphQL API. They share the API schema and query structure, but use
different store hosts, market/language context, and sometimes public Storefront
tokens. EU and Poland share a store host. Korea uses Cafe24 REST instead and did
not return product data without an allowed client ID.

These are live, read-only sample results, not a completed integration or proof
of full catalogue coverage. Existing offer downloads still use HTML.

## Tested endpoints

For every Shopify host below, the tested endpoint is:

```text
POST https://<host>/api/2026-10/graphql.json
Content-Type: application/json
```

“No token” means the request returned products without an authentication header.
“Public token” means the request used an `X-Shopify-Storefront-Access-Token`
published in Anker's browser JavaScript, not a private or Admin API credential.
Currency describes the sampled response, not every supported market.

| Retailer          | Storefront             | Shopify API host                          | Access tested                      | Sample currency |
| ----------------- | ---------------------- | ----------------------------------------- | ---------------------------------- | --------------- |
| `ankerau`         | `www.anker.com/au`     | `oneanker-au.myshopify.com`               | No token                           | AUD             |
| `ankerca`         | `www.anker.com/ca`     | `anker-canada.myshopify.com`              | No token                           | CAD             |
| `ankercom`        | `www.anker.com`        | `ankerus.myshopify.com`                   | No token                           | USD             |
| `ankerde`         | `www.anker.com/de`     | `oneanker-de.myshopify.com`               | No token                           | EUR             |
| `ankereu`         | `www.anker.com/eu-en`  | `ankereu.myshopify.com`                   | No token                           | EUR             |
| `ankerfr`         | `www.anker.com/fr`     | `ankerfra.myshopify.com`                  | No token                           | EUR             |
| `ankeritalycom`   | `www.ankeritaly.com`   | `anker-italy.myshopify.com`               | No token                           | EUR             |
| `ankerjapancom`   | `www.ankerjapan.com`   | `anker-japan-official-site.myshopify.com` | No token                           | JPY             |
| `ankernordicscom` | `www.ankernordics.com` | `ankerneu.myshopify.com`                  | No token                           | SEK             |
| `ankernz`         | `www.anker.com/nz`     | `ankernz.myshopify.com`                   | Public token                       | USD             |
| `ankerpl`         | `www.anker.com/eu-pl`  | `ankereu.myshopify.com`                   | No token, Polish context           | PLN             |
| `ankeruk`         | `www.anker.com/uk`     | `oneanker-uk.myshopify.com`               | No token                           | GBP             |
| `ankervn`         | `www.anker.com/vn`     | `ankervnm.myshopify.com`                  | Public token                       | VND             |
| `ankerkr`         | `ankerkorea.co.kr`     | Not Shopify; see Cafe24 below             | Rejected without allowed client ID | Not verified    |

## Product fields and reproducible query

The successful queries requested product IDs, handles, titles, descriptions,
images, variant IDs, SKUs, barcodes, purchase availability, option names/values,
prices, and comparison prices. Some fields are nullable or empty: for example,
the Nordics sample had no description, and several samples had no barcode or
comparison price. Variant options can supply colors without extracting HTML.

This query reproduces the Australian sample and works with the other Shopify
hosts above, subject to their token and context requirements:

```bash
curl --fail-with-body 'https://oneanker-au.myshopify.com/api/2026-10/graphql.json' \
  -H 'Content-Type: application/json' \
  --data-binary @- <<'JSON'
{
  "query": "{ products(first: 1) { nodes { id handle title description images(first: 1) { nodes { url } } variants(first: 2) { nodes { id sku barcode availableForSale selectedOptions { name value } price { amount currencyCode } compareAtPrice { amount currencyCode } } } } pageInfo { hasNextPage endCursor } } }"
}
JSON
```

The Australian sample returned handle `a1902-100w-power-bank-charging-station`,
SKU `A1902T11`, barcode `194644145453`, and a variant price of AUD 169.95.
These are observations from the investigation date, not fixed product values.

The sample deliberately limits products, variants, and images. An integration
must paginate each connection rather than treating those limits as complete
data. Shopify supports product lookup by handle as well as catalogue listing.
See the [Storefront API reference](https://shopify.dev/docs/api/storefront/latest)
and [product query](https://shopify.dev/docs/api/storefront/latest/queries/product).

## Store specific behavior

### Poland and EU

Both use `ankereu.myshopify.com`. A default request returned English text and
EUR. The same query prefixed with the following operation returned Polish text
and PLN:

```graphql
query @inContext(country: PL, language: PL) {
  products(first: 1) {
    nodes {
      handle
      title
      variants(first: 2) {
        nodes { price { amount currencyCode } }
      }
    }
  }
}
```

For sampled handle `a2637`, Polish context returned PLN 99.00; German country
context with English language returned EUR 26.99. Store identity alone therefore
does not determine the correct localized offer. Preserve separate retailer
identities and explicit market/language settings. Canadian French and other
additional language combinations were not separately tested.

### New Zealand and Vietnam

Without a token, New Zealand returned HTTP 200 with an empty product connection,
including with `country: NZ`. Vietnam returned HTTP 400 with
`Online Store channel is locked.` Both returned HTTP 200 and products when the
request included the public Storefront token used by Anker's browser frontend.

The token maps were present in this
[Anker frontend bundle](https://www.anker.com/_next/static/chunks/pages/_app-0254883c53498edc.js).
This deployment-specific URL and its public tokens can change. Token values are
not copied into this document. Use the public Storefront header for these
requests; no Admin API credentials are needed for the tested queries.

The NZ sample `a1657-maggo-10000mah-power-bank` returned USD 149.95 and
`availableForSale: false`. The NZ page-data response also reported USD. Do not
label these amounts NZD or infer currency from the retailer name. NZ-country
context with a public token has not been verified.

The Vietnam sample `a2544` returned VND 1,400,000 and
`availableForSale: false`. A returned product is not necessarily currently
purchasable.

### Japan JSON alternatives

Both of these returned product JSON without a token:

- [Product listing](https://www.ankerjapan.com/products.json?limit=1):
  `GET https://www.ankerjapan.com/products.json?limit=1`.
- [Individual product](https://www.ankerjapan.com/products/a1621.js):
  `GET https://www.ankerjapan.com/products/a1621.js`.

The individual-product route is Shopify's documented
[Ajax Product API](https://shopify.dev/docs/api/ajax/reference/product).
Keep its monetary representation separate from GraphQL money values.
The equivalent `/products.json?limit=1` request on each of the other 12
Shopify storefront URLs in the table returned HTTP 404 HTML. The presence of a
Shopify backend does not mean its custom storefront exposes Shopify theme routes.

### Next.js page data fallback

New Zealand and Vietnam also returned product JSON via the storefront's
Next.js data route, without a Storefront token:

```text
https://www.anker.com/_next/data/7y_v_RxGcjxROidTJmWZC/nz/products/bundle-a2668t21-2.json
https://www.anker.com/_next/data/7y_v_RxGcjxROidTJmWZC/vn/products/a1642.json
```

Both returned HTTP 200 with `pageProps.product`, including variants, images,
prices, options, and metafields. This avoids parsing a product HTML document,
but depends on a deployment-specific build ID and Anker's internal page schema.
It is a fallback candidate, not a stable public API contract; freshness and
completeness still need validation.

### Korea Cafe24 API

The live Korean storefront identifies its Cafe24 mall as `ankerkr`. The request
to [its product endpoint](https://ankerkr.cafe24api.com/api/v2/products?limit=1)
returned HTTP 401:

```json
{"error":{"code":401,"message":"API is not available. This is not an allowed client_id."}}
```

Cafe24 documents [Front API setup](https://developers.cafe24.com/en/app/front/app/develop/api/frontapi)
and its [REST API](https://developers.cafe24.com/docs/en/api/). An allowed app/client
configuration is needed before this can be treated as a usable product source.
No unauthenticated product-data replacement was confirmed for Korea.

## Integration recommendation and remaining checks

Use one shared Shopify Storefront downloader/query implementation for the 13
Shopify-backed retailers, including Australia, with per-retailer store host,
market/language context, and optional public Storefront token. Keep Korea
separate. This is a recommendation; no download or extraction code was changed
as part of the investigation.

Before replacing HTML extraction:

- Compare several known storefront products against API responses, including
  bundles, discounts, sold-out products, variant IDs, SKU/barcode values, and
  localized prices. A one-product catalogue sample does not establish full
  publication coverage or parity with the website's offers.
- Verify pagination for products, variants, and images, and map handles back
  to the correct regional storefront URL rather than the API hostname.
- Validate missing descriptions, nullable fields, comparison pricing, and
  availability semantics. A purchase flag does not establish inventory count
  or item condition.
- Check the missing rich content independently: Anker's `subtitle`,
  `productInfos`, and `components.htmlContainer` were not established as
  available through these basic GraphQL queries. Exposed metafields or a
  separate content source may still be necessary.
- Recheck current domains and token requirements before implementation.
  Older saved pages and frontend bundles can reference previous store hosts;
  the table records the hosts that returned the sampled data in this audit.

The existing HTML pipeline and source-field behavior are described in the
[Anker crate README](../lib/retailers_anker/README.md) and
[offer facts source coverage](offer-facts-source-coverage.md).
