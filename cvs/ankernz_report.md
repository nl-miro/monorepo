# ankernz language variants within one retailer

**No confirmed same-offer pair in different languages was identified within this file.** This report examines only [ankernz.csv](ankernz.csv), containing 260 records. It does not match against any other retailer file.

## Language evidence

All 260 rows declare `en`. Embedded content contains 260 entries with language counts `{'en': 260}`. Content-array lengths are `{1: 260}`. There is no second language entry attached to an individual offer.

Inspected titles and description excerpts are English. The `/nz/` URL path is a market route, not a second language. All recorded currencies are USD; preserve that stored value if comparing commercial terms. Barcode reuse connects bundles and standalone components.

URL hosts: `{'www.anker.com': 260}`. First path segments: `{'nz': 260}`. No alternative locale route appears in this snapshot.

## Measured matching approaches

A group is one repeated nonempty key. Rows count records participating in those groups; pairs count unordered combinations inside a group. Methods overlap and must not be added. All counts are within this CSV. No pair has different declared row languages; actual text can still differ in language.

| Method                   | Groups | Rows | Pairs |
| ------------------------ | ------ | ---- | ----- |
| Exact Shopify variant ID | 0      | 0    | 0     |
| Exact SKU                | 0      | 0    | 0     |
| Exact SKU plus GTIN      | 0      | 0    | 0     |
| Exact GTIN               | 6      | 13   | 8     |
| Exact product ID         | 45     | 195  | 607   |
| Exact handle             | 45     | 195  | 607   |
| Exact URL                | 0      | 0    | 0     |
| Normalized title         | 47     | 199  | 609   |
| Normalized description   | 50     | 209  | 632   |
| Five-character SKU stem  | 51     | 190  | 456   |

Exact identifiers use stored strings and ignore blanks. Text normalization removes scripts/styles and HTML, decodes entities, collapses whitespace and case-folds. The five-character stem is a broad heuristic for ordinary eight-character A/B/T SKUs; it excludes bundles and is not a verified model-code decoder.

Variant IDs provide the best direct within-store identity key, but none repeats here. Product IDs and handles group variants, so they are candidate generators rather than exact-offer keys. Exact SKU plus GTIN is useful corroboration, with quantity, condition and accessories checked separately.

## Concrete within-file comparisons

Record numbers below are one-based CSV data-record positions, excluding the header. Links are recorded source URLs, not live checks.

### Records 21 and 219

[21: Anker Prime 20,000mAh Power Bank (200W) and 737 Charger (GaNPrime 120W)](https://www.anker.com/nz/products/bundle-a1336011-1-a2148t12-1) and [219: Anker 737 Prime Charger (GaN, 120W, 3-Port)](https://www.anker.com/nz/products/a2148).

| Field                         | Record 21                                   | Record 219                                  |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | BUNDLE-A1336011-1-A2148T12-1                | A2148T12                                    |
| gtin                          | 194644099008                                | 194644099008                                |
| shopify_product_variation_gid | gid://shopify/ProductVariant/45193969533099 | gid://shopify/ProductVariant/45193914286251 |

A power bank plus charger bundle shares the charger barcode with a standalone charger. Both are English; reject as an identical-offer match.

### Records 24 and 88

[24: 2 × Anker 511 Charger (Nano 3, 30W) and Anker 322 3ft USB-C to USB-C Cable](https://www.anker.com/nz/products/bundle-a2147t11-2-a81f6h11-1) and [88: Anker 511 Charger (Nano 3, 30W)](https://www.anker.com/nz/products/a2147?variant=45193979953323).

| Field                         | Record 24                                   | Record 88                                   |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | BUNDLE-A2147T11-2-A81F5H11-1                | A2147T11                                    |
| gtin                          | 194644102838                                | 194644102838                                |
| shopify_product_variation_gid | gid://shopify/ProductVariant/45194001842347 | gid://shopify/ProductVariant/45193979953323 |

A two-charger-and-cable bundle shares a barcode with a single charger. Package quantity and contents differ.

### Records 155 and 164

[155: Anker for Magsafe Powerbank, 321 MagGo Battery (PowerCore Magnetic), 5,000mAh Magnetic Wireless Portable Charger](https://www.anker.com/nz/products/a1616?variant=45193984802987) and [164: 2 x Anker 321 MagGo Battery (PowerCore Magnetic 5K)](https://www.anker.com/nz/products/bundle-a1616011-1-a1616021-1).

| Field                         | Record 155                                  | Record 164                                  |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | A1616011                                    | BUNDLE-A1616011-1-A1616021-1                |
| gtin                          | 194644124830                                | 194644124830                                |
| shopify_product_variation_gid | gid://shopify/ProductVariant/45193984802987 | gid://shopify/ProductVariant/45193952231595 |

A single battery and a two-pack share a barcode and description. Reused text is not identity or translation evidence.

## Detection procedure for this retailer

1. Group records within this retailer by exact variant ID, then full SKU plus nonempty GTIN. Keep all original record IDs and listing URLs. Separately review GTIN-only and model-stem candidates; do not merge them automatically.
2. Detect language from cleaned title and description independently, retaining the declared language as a separate field. Use long descriptive prose and allow mixed or unknown results; product names and model codes are poor language signals.
3. Require two identity-compatible records with different observed content languages before calling them language variants. Compare capacity, wattage, ports, length, color, plug, pack count, accessories and condition. Equal price is supporting context, not proof.
4. Where identifiers are missing or regional suffixes differ, translate descriptions to a common language or use multilingual similarity to propose candidates. Preserve numerical specifications and send conflicts for review. No translation service, embedding model or automated language classifier was run in this analysis.

The measured work comprises identifier/text grouping and inspection of the cited descriptions and duplicate groups. It does not establish that this retailer lacks other-language pages outside the supplied snapshot. If more locale exports become available for this same retailer, append them with an explicit locale field and rerun the same within-retailer procedure.
