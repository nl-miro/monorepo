# ankerpl language variants within one retailer

**One English–Polish product candidate was identified; exact-offer equivalence remains unresolved.** This report examines only [ankerpl.csv](ankerpl.csv), containing 206 records. It does not match against any other retailer file.

## Language evidence

All 206 rows declare `pl`. Embedded content contains 206 entries with language counts `{'pl': 206}`. Content-array lengths are `{1: 206}`. There is no second language entry attached to an individual offer.

Actual content is mixed despite every row being labeled `pl`. Record 95 has English prose beginning “Ultra-High Cell Capacity”; record 120 has its Polish counterpart beginning “Ultra-wysoka pojemność ogniwa”. They share a barcode and product model but have different full SKUs and variant IDs. This is a concrete multilingual product candidate, not a confirmed duplicate of the exact same offer. Record 2 also contains English prose (“Power for 2”) despite its Polish label.

URL hosts: `{'www.anker.com': 206}`. First path segments: `{'eu-pl': 206}`. No alternative locale route appears in this snapshot.

## Measured matching approaches

A group is one repeated nonempty key. Rows count records participating in those groups; pairs count unordered combinations inside a group. Methods overlap and must not be added. All counts are within this CSV. No pair has different declared row languages; actual text can still differ in language.

| Method                   | Groups | Rows | Pairs |
| ------------------------ | ------ | ---- | ----- |
| Exact Shopify variant ID | 0      | 0    | 0     |
| Exact SKU                | 16     | 32   | 16    |
| Exact SKU plus GTIN      | 16     | 32   | 16    |
| Exact GTIN               | 11     | 51   | 161   |
| Exact product ID         | 41     | 137  | 353   |
| Exact handle             | 41     | 137  | 353   |
| Exact URL                | 0      | 0    | 0     |
| Normalized title         | 41     | 140  | 476   |
| Normalized description   | 38     | 139  | 881   |
| Five-character SKU stem  | 38     | 102  | 105   |

Exact identifiers use stored strings and ignore blanks. Text normalization removes scripts/styles and HTML, decodes entities, collapses whitespace and case-folds. The five-character stem is a broad heuristic for ordinary eight-character A/B/T SKUs; it excludes bundles and is not a verified model-code decoder.

Variant IDs provide the best direct within-store identity key, but none repeats here. Product IDs and handles group variants, so they are candidate generators rather than exact-offer keys. Exact SKU plus GTIN is useful corroboration, with quantity, condition and accessories checked separately.

## Concrete within-file comparisons

Record numbers below are one-based CSV data-record positions, excluding the header. Links are recorded source URLs, not live checks.

### Records 95 and 120

[95: Anker 325 Power Bank (PowerCore 20K)](https://www.anker.com/eu-pl/products/a1268013) and [120: Powerbank Anker 325 (PowerCore 20K)](https://www.anker.com/eu-pl/products/a1268).

| Field                         | Record 95                                   | Record 120                                  |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | A1268013                                    | A1268015                                    |
| gtin                          | 848061059861                                | 848061059861                                |
| shopify_product_variation_gid | gid://shopify/ProductVariant/54855592477048 | gid://shopify/ProductVariant/41510216335550 |

English and Polish descriptions cover the same 20,000mAh model, PowerIQ/VoltageBoost, dual USB outputs and similar charging claims. Both store price amount 17900 PLN and sold_out. Full SKUs A1268013/A1268015 and Shopify product/variant IDs differ: investigate variant/package differences before accepting an exact-offer match. Metadata-only language filtering would miss this candidate.

### Records 122 and 190

[122: Inteligentna ładowarka z wyświetlaczem Anker Nano 45 W (1-Zestaw)](https://www.anker.com/eu-pl/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049034616) and [190: Ładowarka Anker Nano (45 W, inteligentny wyświetlacz)](https://www.anker.com/eu-pl/products/a121d-45w-usb-c-fast-charger?variant=55532885901688).

| Field                         | Record 122                                  | Record 190                                  |
| ----------------------------- | ------------------------------------------- | ------------------------------------------- |
| sku                           | BUNDLE-A121D311-2                           | BUNDLE-A121D311-2                           |
| gtin                          | 194644332037                                | 194644332037                                |
| shopify_product_variation_gid | gid://shopify/ProductVariant/55534049034616 | gid://shopify/ProductVariant/55532885901688 |

Same bundle SKU and GTIN, but both titles and identical descriptions are Polish. The title mentioning “1-Zestaw” also conflicts with the SKU suffix `-2`; verify quantity rather than treating naming differences as translation.

## Repeated SKU inventory

These are identity-review candidates. None is confirmed as a different-language pair.

| SKU                          | Record positions | Same raw description | Same GTIN |
| ---------------------------- | ---------------- | -------------------- | --------- |
| BUNDLE-A121D311-2            | 122, 190         | True                 | True      |
| BUNDLE-A121D331-2            | 123, 191         | True                 | True      |
| BUNDLE-A121D3O1-2            | 124, 138         | True                 | True      |
| BUNDLE-A121D311-1-A121D321-1 | 125, 139         | True                 | True      |
| BUNDLE-A121D331-1-A121D321-1 | 126, 136         | True                 | True      |
| BUNDLE-A121D311-1-A121D331-1 | 127, 137         | True                 | True      |
| BUNDLE-A121D3O1-1-A121D331-1 | 128, 141         | True                 | True      |
| BUNDLE-A121D3O1-1-A121D321-1 | 129, 142         | True                 | True      |
| BUNDLE-A121D331-1-A82E2012-1 | 140, 200         | True                 | True      |
| BUNDLE-A121D3O1-1-A121D311-1 | 143, 150         | True                 | True      |
| BUNDLE-A121D321-1-A82E2021-1 | 144, 196         | True                 | True      |
| BUNDLE-A121D311-1-A82E2011-1 | 145, 197         | True                 | True      |
| BUNDLE-A121D331-1-A82E2011-1 | 146, 198         | True                 | True      |
| BUNDLE-A121D321-1-A82E2022-1 | 147, 192         | True                 | True      |
| BUNDLE-A121D311-1-A82E2012-1 | 148, 199         | True                 | True      |
| BUNDLE-A121D321-2            | 166, 194         | True                 | True      |

## Detection procedure for this retailer

1. Group records within this retailer by exact variant ID, then full SKU plus nonempty GTIN. Keep all original record IDs and listing URLs. Separately review GTIN-only and model-stem candidates; do not merge them automatically.
2. Detect language from cleaned title and description independently, retaining the declared language as a separate field. Use long descriptive prose and allow mixed or unknown results; product names and model codes are poor language signals.
3. Require two identity-compatible records with different observed content languages before calling them language variants. Compare capacity, wattage, ports, length, color, plug, pack count, accessories and condition. Equal price is supporting context, not proof.
4. Where identifiers are missing or regional suffixes differ, translate descriptions to a common language or use multilingual similarity to propose candidates. Preserve numerical specifications and send conflicts for review. No translation service, embedding model or automated language classifier was run in this analysis.

The measured work comprises identifier/text grouping and inspection of the cited descriptions and duplicate groups. It does not establish that this retailer lacks other-language pages outside the supplied snapshot. If more locale exports become available for this same retailer, append them with an explicit locale field and rerun the same within-retailer procedure.
