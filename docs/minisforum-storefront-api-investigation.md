# Minisforum storefront product API investigation

Investigation date: 2026-11-07. Covers all 10 registered Minisforum retailer codes.

All 10 storefronts returned products through Shopify Storefront GraphQL without an authentication header. Each also returned a JSON product listing and an individual-product Ajax response. These stores use the same API family as the Shopify-backed Anker stores, with different store hosts and market/language context.

These are live sample results, not a completed integration or a full catalogue-parity check. Existing HTML download and extraction code remains unchanged.

## Tested endpoints

Every Shopify host below accepted this endpoint and returned HTTP 200, a nonempty product connection, and no GraphQL errors:

```text
POST https://<host>/api/2026-10/graphql.json
Content-Type: application/json
```

Currencies are those of the default sampled response, not all supported market currencies.

| Retailer       | Live storefront      | Shopify API host                    | Default currency |
| -------------- | -------------------- | ----------------------------------- | ---------------- |
| `minisforumau` | au.minisforum.com    | `j94nqi-71.myshopify.com`           | AUD              |
| `minisforumca` | ca.minisforum.com    | `seaminisforum.myshopify.com`       | CAD              |
| `minisforumeu` | minisforumpc.eu      | `minisforum-eu.myshopify.com`       | EUR              |
| `minisforumfr` | minisforumpc.fr      | `minisforum-fr.myshopify.com`       | EUR              |
| `minisforumhk` | minisforum.hk        | `minisforumcn.myshopify.com`        | HKD              |
| `minisforumjp` | www.minisforum.jp    | `minisforum-jp.myshopify.com`       | JPY              |
| `minisforumkr` | www.minisforum.kr    | `minisforumkr.myshopify.com`        | KRW              |
| `minisforumru` | minisforum.ru        | `store-minisforum-ru.myshopify.com` | USD              |
| `minisforumuk` | www.minisforum.uk    | `minisforum-uk.myshopify.com`       | GBP              |
| `minisforumus` | store.minisforum.com | `minisforum.myshopify.com`          | USD              |

## JSON alternatives

Every live storefront in the table also returned HTTP 200 with product JSON for both routes below, without authentication:

```text
GET https://<live-storefront>/products.json?limit=1
GET https://<live-storefront>/products/<returned-handle>.js
```

The Ajax handle came from the listing response, rather than being guessed. Listing and GraphQL samples can differ. The [probe results](minisforum-ugreen-api-probe-results.json) record exact URLs, sample handles, statuses, and Ajax variant counts.

The individual-product route is Shopify's documented [Ajax Product API](https://shopify.dev/docs/api/ajax/reference/product). Treat the listing route as observed storefront behavior rather than a substitute for the documented, versioned GraphQL catalogue contract.

## Reproducible query

Replace `HOST` with a Shopify host from the table. No token was needed for this basic query during the investigation.

```bash
curl --fail-with-body 'https://HOST/api/2026-10/graphql.json' \
  -H 'Content-Type: application/json' \
  --data-binary @- <<'JSON'
{
  "query": "{ products(first: 1) { nodes { id handle title description images(first: 1) { nodes { url } } variants(first: 2) { nodes { id sku barcode availableForSale selectedOptions { name value } price { amount currencyCode } compareAtPrice { amount currencyCode } } } } pageInfo { hasNextPage endCursor } } }"
}
JSON
```

The query requests product and variant IDs, handles, titles, descriptions, images, SKUs, barcodes, purchase availability, option names/values, prices, and comparison prices. Fields may be null or empty. The sample limits are deliberately small: production requests must paginate products, variants, and images independently. See the [Storefront API reference](https://shopify.dev/docs/api/storefront/latest) and [product lookup by handle](https://shopify.dev/docs/api/storefront/latest/queries/product).

## Store specific findings

- **Russia returned USD.** Default GraphQL, `GET https://minisforum.ru/cart.js`, and GraphQL with `@inContext(country: RU)` all returned USD. Handle `minisforum-um890-pro` returned USD 463.00 under RU context. Do not infer RUB from the domain.
- **Korea is Shopify, unlike Anker Korea.** The configured `https://minisforum.kr` redirects to `https://www.minisforum.kr`. GraphQL and `/cart.js` returned KRW.
- **Australia has an opaque Shopify hostname:** `j94nqi-71.myshopify.com`. Do not derive backend names from the brand.
- **Catalogues include service and adjustment products.** The first GraphQL samples included `price-difference` for EU, `exclusive-use-link` for Japan, and `tariff-insurance` for UK. Japan and UK listing samples were `shipping-protection`, with 98 variants each in Ajax. These must not automatically become hardware offers.
- **Descriptions can be empty.** AU, CA, FR, HK and US GraphQL samples lacked descriptions. Basic API product data does not guarantee all marketing copy from the page.

## Integration recommendation and remaining checks

Use a common Shopify Storefront downloader with per-retailer host, country, and language configuration. Ajax is a useful alternative for known handles. Keep its monetary representation and currency context separate from GraphQL money values; do not reuse HTML-parser price scaling assumptions, particularly for JPY/KRW.

- Compare several real hardware products against current structuralized output, including all variants, bundles, discounts, deposits, refurbished items and sold-out products where applicable.
- Start validation with known product handles. Catalogue listing can include insurance, gift cards, adjustment products, or items outside the current sitemap-derived offer set.
- Verify publication coverage and paginate all relevant connections before replacing sitemap discovery or per-page downloads.
- Preserve source IDs, observed currency, exact decimal amounts, and separate retailer identities. SKU/barcode values may be absent.
- Do not interpret `availableForSale` as an inventory count or item condition. Comparison prices may be null.
- Validate rich descriptions, specifications, custom metafields, and media independently. Tokenless access to this query does not establish access to every Storefront API field.
- Map product handles to regional customer-facing storefront URLs, not the Shopify API hostname.

Related investigations: [Anker including AU](anker-storefront-api-investigation.md), [Minisforum](minisforum-storefront-api-investigation.md), and [Ugreen](ugreen-storefront-api-investigation.md).

