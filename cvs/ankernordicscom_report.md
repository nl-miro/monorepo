# ankernordicscom language variants within one retailer

**No confirmed same-offer pair in different languages was identified within this file.** This report examines only [ankernordicscom.csv](ankernordicscom.csv), containing 148 records. It does not match against any other retailer file.

## Language evidence

All 148 rows declare `en`. Embedded content contains 148 entries with language counts `{'en': 148}`. Content-array lengths are `{1: 148}`. There is no second language entry attached to an individual offer.

Inspected title and description excerpts are English. The file has SE market metadata and SEK currency, but those market fields are not language evidence. Both barcode duplicate groups pair refurbished listings with ordinary listings, not language versions.

URL hosts: `{'www.ankernordics.com': 148}`. First path segments: `{'products': 148}`. No alternative locale route appears in this snapshot.

## Measured matching approaches

A group is one repeated nonempty key. Rows count records participating in those groups; pairs count unordered combinations inside a group. Methods overlap and must not be added. All counts are within this CSV. No pair has different declared row languages; actual text can still differ in language.

| Method                   | Groups | Rows | Pairs |
| ------------------------ | ------ | ---- | ----- |
| Exact Shopify variant ID | 0      | 0    | 0     |
| Exact SKU                | 0      | 0    | 0     |
| Exact SKU plus GTIN      | 0      | 0    | 0     |
| Exact GTIN               | 2      | 4    | 2     |
| Exact product ID         | 17     | 55   | 74    |
| Exact handle             | 17     | 55   | 74    |
| Exact URL                | 0      | 0    | 0     |
| Normalized title         | 17     | 55   | 74    |
| Normalized description   | 23     | 72   | 109   |
| Five-character SKU stem  | 11     | 32   | 38    |

Exact identifiers use stored strings and ignore blanks. Text normalization removes scripts/styles and HTML, decodes entities, collapses whitespace and case-folds. The five-character stem is a broad heuristic for ordinary eight-character A/B/T SKUs; it excludes bundles and is not a verified model-code decoder.

Variant IDs provide the best direct within-store identity key, but none repeats here. Product IDs and handles group variants, so they are candidate generators rather than exact-offer keys. Exact SKU plus GTIN is useful corroboration, with quantity, condition and accessories checked separately.

## Concrete within-file comparisons

Record numbers below are one-based CSV data-record positions, excluding the header. Links are recorded source URLs, not live checks.

### Records 11 and 40

[11: eufy Baby Monitor E21 (refurbished)](https://www.ankernordics.com/products/eufy-baby-monitor-e21-f0) and [40: eufy Baby Monitor E21](https://www.ankernordics.com/products/eufy-baby-monitor-e21).

| Field                         | Record 11                                   | Record 40                                   |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | E8354321-F0                                 | E8354321                                    |
| gtin                          | 194644285876                                | 194644285876                                |
| shopify_product_variation_gid | gid://shopify/ProductVariant/51323230781718 | gid://shopify/ProductVariant/50046537859350 |

Same GTIN, but one title explicitly says refurbished and its SKU has the `-F0` suffix. Both descriptions are English. Retain separate condition-specific offers.

### Records 85 and 120

[85: soundcore P40i \| Smart ANC True-Wireless Earbuds (refurbished)](https://www.ankernordics.com/products/p40i-f0?variant=51323229339926) and [120: soundcore P40i \| Smart ANC True-Wireless Earbuds](https://www.ankernordics.com/products/p40i-a3955011?variant=49663279104278).

| Field                         | Record 85                                   | Record 120                                  |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | A3955GQ1-F0                                 | A3955GQ1                                    |
| gtin                          | 194644190958                                | 194644190958                                |
| shopify_product_variation_gid | gid://shopify/ProductVariant/51323229339926 | gid://shopify/ProductVariant/49663279104278 |

The same refurbished-versus-ordinary distinction occurs for soundcore P40i. Shared barcode or copied description must not override condition.

## Detection procedure for this retailer

1. Group records within this retailer by exact variant ID, then full SKU plus nonempty GTIN. Keep all original record IDs and listing URLs. Separately review GTIN-only and model-stem candidates; do not merge them automatically.
2. Detect language from cleaned title and description independently, retaining the declared language as a separate field. Use long descriptive prose and allow mixed or unknown results; product names and model codes are poor language signals.
3. Require two identity-compatible records with different observed content languages before calling them language variants. Compare capacity, wattage, ports, length, color, plug, pack count, accessories and condition. Equal price is supporting context, not proof.
4. Where identifiers are missing or regional suffixes differ, translate descriptions to a common language or use multilingual similarity to propose candidates. Preserve numerical specifications and send conflicts for review. No translation service, embedding model or automated language classifier was run in this analysis.

The measured work comprises identifier/text grouping and inspection of the cited descriptions and duplicate groups. It does not establish that this retailer lacks other-language pages outside the supplied snapshot. If more locale exports become available for this same retailer, append them with an explicit locale field and rerun the same within-retailer procedure.
