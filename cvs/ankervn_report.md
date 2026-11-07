# ankervn language variants within one retailer

**No confirmed same-offer pair in different languages was identified within this file.** This report examines only [ankervn.csv](ankervn.csv), containing 218 records. It does not match against any other retailer file.

## Language evidence

All 218 rows declare `vi`. Embedded content contains 218 entries with language counts `{'vi': 218}`. Content-array lengths are `{1: 218}`. There is no second language entry attached to an individual offer.

Descriptions in inspected duplicate-SKU groups are Vietnamese. Record 61 (`A3320011`) instead has English prose beginning “AnkerWork M650 is an all-in-one wireless lavalier mic”, despite its `vi` label. Its SKU, GTIN, product ID and five-character model stem have no second record in this file. Record 134 mixes an English heading (“Swift Power Anywhere”) with Vietnamese prose. These language issues do not establish another-language copy of the same offer.

URL hosts: `{'www.anker.com': 218}`. First path segments: `{'vn': 218}`. No alternative locale route appears in this snapshot.

## Measured matching approaches

A group is one repeated nonempty key. Rows count records participating in those groups; pairs count unordered combinations inside a group. Methods overlap and must not be added. All counts are within this CSV. No pair has different declared row languages; actual text can still differ in language.

| Method                   | Groups | Rows | Pairs |
| ------------------------ | ------ | ---- | ----- |
| Exact Shopify variant ID | 0      | 0    | 0     |
| Exact SKU                | 15     | 30   | 15    |
| Exact SKU plus GTIN      | 15     | 30   | 15    |
| Exact GTIN               | 15     | 30   | 15    |
| Exact product ID         | 45     | 188  | 504   |
| Exact handle             | 45     | 188  | 504   |
| Exact URL                | 0      | 0    | 0     |
| Normalized title         | 45     | 188  | 504   |
| Normalized description   | 45     | 188  | 504   |
| Five-character SKU stem  | 52     | 188  | 396   |

Exact identifiers use stored strings and ignore blanks. Text normalization removes scripts/styles and HTML, decodes entities, collapses whitespace and case-folds. The five-character stem is a broad heuristic for ordinary eight-character A/B/T SKUs; it excludes bundles and is not a verified model-code decoder.

Variant IDs provide the best direct within-store identity key, but none repeats here. Product IDs and handles group variants, so they are candidate generators rather than exact-offer keys. Exact SKU plus GTIN is useful corroboration, with quantity, condition and accessories checked separately.

## Concrete within-file comparisons

Record numbers below are one-based CSV data-record positions, excluding the header. Links are recorded source URLs, not live checks.

### Records 143 and 163

[143: Cáp Anker PowerLine III USB-C sang USB-C](https://www.anker.com/vn/products/a8852?variant=41695773917265) and [163: Cáp Anker 541 USB-C sang USB-C](https://www.anker.com/vn/products/a8854?variant=41695773818961).

| Field                         | Record 143                                  | Record 163                                  |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | A8851011                                    | A8851011                                    |
| gtin                          | 194644030032                                | 194644030032                                |
| shopify_product_variation_gid | gid://shopify/ProductVariant/41695773917265 | gid://shopify/ProductVariant/41695773818961 |

Same SKU and GTIN under PowerLine III and 541 naming, with different product/variant IDs. The descriptions differ but remain Vietnamese. This is an alias/listing candidate, not another-language offer.

### Records 187 and 194

[187: Cáp Anker 543 USB-C sang USB-C (Bện sinh học)](https://www.anker.com/vn/products/a80e6?variant=41695767920721) and [194: Cáp Anker 543 USB C sang USB C (240W, 10 ft)](https://www.anker.com/vn/products/a80e7).

| Field                         | Record 187                                  | Record 194                                  |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | A80E7011                                    | A80E7011                                    |
| gtin                          | 194644136345                                | 194644136345                                |
| shopify_product_variation_gid | gid://shopify/ProductVariant/41695767920721 | gid://shopify/ProductVariant/41695766184017 |

Same SKU and GTIN under generic braided-cable and explicit 240W/10 ft titles. Both descriptions are Vietnamese; verify length/specifications before merging listings.

## Repeated SKU inventory

These are identity-review candidates. None is confirmed as a different-language pair.

| SKU      | Record positions | Same raw description | Same GTIN |
| -------- | ---------------- | -------------------- | --------- |
| A8851011 | 143, 163         | False                | True      |
| A8853021 | 144, 159         | False                | True      |
| A8851021 | 146, 160         | False                | True      |
| A8853011 | 162, 172         | False                | True      |
| A86620Q1 | 179, 213         | False                | True      |
| A8662051 | 182, 209         | False                | True      |
| A8663011 | 183, 206         | False                | True      |
| A8663021 | 185, 208         | False                | True      |
| A8663061 | 186, 212         | False                | True      |
| A80E7011 | 187, 194         | False                | True      |
| A86630Q1 | 197, 214         | False                | True      |
| A8662021 | 199, 207         | False                | True      |
| A8662061 | 200, 211         | False                | True      |
| A8662011 | 201, 205         | False                | True      |
| A8663051 | 203, 210         | False                | True      |

## Detection procedure for this retailer

1. Group records within this retailer by exact variant ID, then full SKU plus nonempty GTIN. Keep all original record IDs and listing URLs. Separately review GTIN-only and model-stem candidates; do not merge them automatically.
2. Detect language from cleaned title and description independently, retaining the declared language as a separate field. Use long descriptive prose and allow mixed or unknown results; product names and model codes are poor language signals.
3. Require two identity-compatible records with different observed content languages before calling them language variants. Compare capacity, wattage, ports, length, color, plug, pack count, accessories and condition. Equal price is supporting context, not proof.
4. Where identifiers are missing or regional suffixes differ, translate descriptions to a common language or use multilingual similarity to propose candidates. Preserve numerical specifications and send conflicts for review. No translation service, embedding model or automated language classifier was run in this analysis.

The measured work comprises identifier/text grouping and inspection of the cited descriptions and duplicate groups. It does not establish that this retailer lacks other-language pages outside the supplied snapshot. If more locale exports become available for this same retailer, append them with an explicit locale field and rerun the same within-retailer procedure.
