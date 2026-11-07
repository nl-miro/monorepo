# Ugreen storefront product API investigation

Investigation date: 2026-11-07. Covers all 21 registered Ugreen retailer codes.

All 21 storefronts returned products through Shopify Storefront GraphQL without an authentication header. Each also returned a JSON product listing and an individual-product Ajax response. These stores use the same API family as the Shopify-backed Anker stores, with different store hosts and market/language context.

These are live sample results, not a completed integration or a full catalogue-parity check. Existing HTML download and extraction code remains unchanged.

## Tested endpoints

Every Shopify host below accepted this endpoint and returned HTTP 200, a nonempty product connection, and no GraphQL errors:

```text
POST https://<host>/api/2026-10/graphql.json
Content-Type: application/json
```

Currencies are those of the default sampled response, not all supported market currencies.

| Retailer | Live storefront | Shopify API host | Default currency |
| --- | --- | --- | --- |
| `ugreenca` | ca.ugreen.com | `ugreen-ca.myshopify.com` | CAD |
| `ugreencom` | www.ugreen.com | `ugreen-brandsite.myshopify.com` | HKD |
| `ugreende` | de.ugreen.com | `ugreen-de.myshopify.com` | EUR |
| `ugreeneu` | eu.ugreen.com | `ugreen-eu.myshopify.com` | EUR |
| `ugreenfr` | fr.ugreen.com | `ugreen-fr.myshopify.com` | EUR |
| `ugreenin` | ugreenindia.com | `990f9c-b7.myshopify.com` | INR |
| `ugreenjp` | ugreen.jp | `ugreen-jp.myshopify.com` | JPY |
| `ugreenkr` | kr.ugreen.com | `ugreen-kr.myshopify.com` | KRW |
| `ugreennas` | ai.ugreen.com | `ugreen-nas.myshopify.com` | USD |
| `ugreennasau` | ai-au.ugreen.com | `nas-ugreen-au.myshopify.com` | AUD |
| `ugreennasca` | ai-ca.ugreen.com | `ugreen-nas-ca.myshopify.com` | CAD |
| `ugreennasde` | ai-de.ugreen.com | `ugreen-de-nas.myshopify.com` | EUR |
| `ugreennases` | ai-es.ugreen.com | `ugreen-nas-es.myshopify.com` | EUR |
| `ugreennaseu` | ai-eu.ugreen.com | `ugreen-nas-eu.myshopify.com` | EUR |
| `ugreennasfr` | ai-fr.ugreen.com | `nas-ugreen-fr.myshopify.com` | EUR |
| `ugreennasit` | ai-it.ugreen.com | `ugreen-nas-it.myshopify.com` | EUR |
| `ugreennasjp` | store.ugreen.jp | `ugreen-nas-jp.myshopify.com` | JPY |
| `ugreennasuk` | ai-uk.ugreen.com | `ugreen-nas-uk.myshopify.com` | GBP |
| `ugreennl` | nl.ugreen.com | `ugreen-nl.myshopify.com` | EUR |
| `ugreenuk` | uk.ugreen.com | `ugreen-uk.myshopify.com` | GBP |
| `ugreenus` | us.ugreen.com | `ugreenmall.myshopify.com` | USD |

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

- **Global and US are different stores.** `ugreencom` uses `ugreen-brandsite.myshopify.com`; `ugreenus` uses `ugreenmall.myshopify.com`. Keep separate retailer identities and catalogues.
- **Global-site prices require explicit context.** Default GraphQL returned HKD. `GET https://www.ugreen.com/cart.js` returned USD, and GraphQL with `@inContext(country: US)` returned USD for the same sampled GraphQL handle `usa-65585` (USD 1301.00). Do not compare default GraphQL prices with storefront/Ajax prices without aligning the market.
- **All ten NAS storefronts redirect to new origins.** The live destinations are recorded below. Keep retailer codes stable and validate product URL redirects separately.

| Retailer | Configured origin | Observed live origin |
| --- | --- | --- |
| `ugreennas` | https://nas.ugreen.com | https://ai.ugreen.com |
| `ugreennasau` | https://nas-au.ugreen.com | https://ai-au.ugreen.com |
| `ugreennasca` | https://nas-ca.ugreen.com | https://ai-ca.ugreen.com |
| `ugreennasde` | https://nas-de.ugreen.com | https://ai-de.ugreen.com |
| `ugreennases` | https://nas-es.ugreen.com | https://ai-es.ugreen.com |
| `ugreennaseu` | https://nas-eu.ugreen.com | https://ai-eu.ugreen.com |
| `ugreennasfr` | https://nas-fr.ugreen.com | https://ai-fr.ugreen.com |
| `ugreennasit` | https://nas-it.ugreen.com | https://ai-it.ugreen.com |
| `ugreennasjp` | https://nas.ugreen.jp | https://store.ugreen.jp |
| `ugreennasuk` | https://nas-uk.ugreen.com | https://ai-uk.ugreen.com |

- **India redirects** from `www.ugreenindia.com` to `ugreenindia.com`, backed by `990f9c-b7.myshopify.com`.
- **Korea and Japan are Shopify.** Their GraphQL samples returned KRW and JPY; the additional Japanese `/cart.js` check returned JPY.
- **Catalogues extend beyond ordinary hardware offers.** Listing samples included a US gift card, a German member-reward product, and a Netherlands mystery box.
- **Descriptions can be empty.** The global, NAS AU and NAS Japan GraphQL samples lacked descriptions.

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

