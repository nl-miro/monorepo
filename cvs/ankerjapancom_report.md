# ankerjapancom language variants within one retailer

**No confirmed same-offer pair in different languages was identified within this file.** This report examines only [ankerjapancom.csv](ankerjapancom.csv), containing 212 records. It does not match against any other retailer file.

## Language evidence

All 212 rows declare `ja`. Embedded content contains 212 entries with language counts `{'ja': 212}`. Content-array lengths are `{1: 212}`. There is no second language entry attached to an individual offer.

Product titles often retain English names while descriptions contain Japanese. For example, records 118 and 170 both say “Anker PowerCore II 20000” and contain Japanese descriptions. English branding is not evidence of an English-language offer. HTML descriptions include CSS; strip script and style blocks before language detection.

URL hosts: `{'www.ankerjapan.com': 212}`. First path segments: `{'products': 212}`. No alternative locale route appears in this snapshot.

## Measured matching approaches

A group is one repeated nonempty key. Rows count records participating in those groups; pairs count unordered combinations inside a group. Methods overlap and must not be added. All counts are within this CSV. No pair has different declared row languages; actual text can still differ in language.

| Method                   | Groups | Rows | Pairs |
| ------------------------ | ------ | ---- | ----- |
| Exact Shopify variant ID | 0      | 0    | 0     |
| Exact SKU                | 0      | 0    | 0     |
| Exact SKU plus GTIN      | 0      | 0    | 0     |
| Exact GTIN               | 2      | 4    | 2     |
| Exact product ID         | 51     | 155  | 195   |
| Exact handle             | 51     | 155  | 195   |
| Exact URL                | 0      | 0    | 0     |
| Normalized title         | 51     | 158  | 203   |
| Normalized description   | 52     | 158  | 198   |
| Five-character SKU stem  | 45     | 147  | 220   |

Exact identifiers use stored strings and ignore blanks. Text normalization removes scripts/styles and HTML, decodes entities, collapses whitespace and case-folds. The five-character stem is a broad heuristic for ordinary eight-character A/B/T SKUs; it excludes bundles and is not a verified model-code decoder.

Variant IDs provide the best direct within-store identity key, but none repeats here. Product IDs and handles group variants, so they are candidate generators rather than exact-offer keys. Exact SKU plus GTIN is useful corroboration, with quantity, condition and accessories checked separately.

## Concrete within-file comparisons

Record numbers below are one-based CSV data-record positions, excluding the header. Links are recorded source URLs, not live checks.

### Records 118 and 170

[118: Anker PowerCore II 20000](https://www.ankerjapan.com/products/a1260s) and [170: Anker PowerCore II 20000](https://www.ankerjapan.com/products/a1260?variant=37214149968033).

| Field                         | Record 118                                  | Record 170                                  |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | missing                                     | A1260011                                    |
| gtin                          | 4571411187162                               | 4571411187162                               |
| shopify_product_variation_gid | gid://shopify/ProductVariant/37282548416673 | gid://shopify/ProductVariant/37214149968033 |

Same GTIN and title, different product/variant IDs; one row has no SKU. Both descriptions are Japanese. This is a duplicate-product review candidate, not a language pair.

### Records 126 and 157

[126: Anker PowerCore 20100](https://www.ankerjapan.com/products/a1271?variant=37209419841697) and [157: Anker PowerCore 20100](https://www.ankerjapan.com/products/a1271s).

| Field                         | Record 126                                  | Record 157                                  |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | A1271032                                    | missing                                     |
| gtin                          | 4571411190230                               | 4571411190230                               |
| shopify_product_variation_gid | gid://shopify/ProductVariant/37209419841697 | gid://shopify/ProductVariant/37282548088993 |

Same GTIN/title, but one SKU is missing and the descriptions differ in Japanese. Review identity and source quality; text differences do not demonstrate translation.

## Detection procedure for this retailer

1. Group records within this retailer by exact variant ID, then full SKU plus nonempty GTIN. Keep all original record IDs and listing URLs. Separately review GTIN-only and model-stem candidates; do not merge them automatically.
2. Detect language from cleaned title and description independently, retaining the declared language as a separate field. Use long descriptive prose and allow mixed or unknown results; product names and model codes are poor language signals.
3. Require two identity-compatible records with different observed content languages before calling them language variants. Compare capacity, wattage, ports, length, color, plug, pack count, accessories and condition. Equal price is supporting context, not proof.
4. Where identifiers are missing or regional suffixes differ, translate descriptions to a common language or use multilingual similarity to propose candidates. Preserve numerical specifications and send conflicts for review. No translation service, embedding model or automated language classifier was run in this analysis.

The measured work comprises identifier/text grouping and inspection of the cited descriptions and duplicate groups. It does not establish that this retailer lacks other-language pages outside the supplied snapshot. If more locale exports become available for this same retailer, append them with an explicit locale field and rerun the same within-retailer procedure.
