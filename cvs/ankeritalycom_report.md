# ankeritalycom same offer language detection

**No different-language version of the same offer was established within this file.** The only repeated SKU, `A8432022`, has two Italian listings. English text occurs in some other fields, but does not establish a duplicate in a second language.

This report uses only [ankeritalycom.csv](ankeritalycom.csv), containing **220 records**. It asks whether that retailer lists the same merchandise in another language and which signals can detect it. Record numbers below are one-based data-row ordinals excluding the header, not physical CSV line numbers. Linked product URLs are snapshot evidence, not live availability checks.

## Language evidence

- Declared row languages: `{'it': 220}`.
- Embedded content languages: `{'it': 220}`; content-entry counts per row: `{1: 220}`.
- Embedded title or description differs from the row fields in **0 rows**. The content arrays therefore provide no extra translated entry.
- Missing descriptions: **0**. Empty text is unknown language, not evidence of translation.
- All URLs use `www.ankeritaly.com/products/`; there is no language path segment.

Language observations below come from inspecting titles and description excerpts, including repeated-SKU groups. No automated language classifier, machine translation or multilingual embedding model was run. Row labels and URL locale alone would miss mixed-language fields.

| Record and SKU                                                                                                                                   | Observed text language                                                                                                                                               |
| ------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [record 102](https://www.ankeritaly.com/products/a3388z11-89): `D1105Z11-86`                                                                     | Italian title “Accessorio a quadrifoglio” with English description beginning “Bring a sparkle of winter charm”.                                                      |
| [record 210](https://www.ankeritaly.com/products/a1638-10k-45w-power-bank?variant=47953293443209): `gid://shopify/ProductVariant/47953293443209` | English title with Italian description beginning “La tua ricarica in movimento”. The SKU field contains a Shopify variant GID rather than a normal merchandise code. |

## Measured matching approaches

A repeated-value group contains at least two records with the same nonempty key. “Rows” counts records in those groups; “pairs” counts unordered record pairs. Methods overlap. Identifier equality uses the stored value without trimming suffixes or GTIN padding. Normalized text strips HTML, decodes entities, collapses whitespace and case-folds.

| Key                    | Repeated groups | Rows | Pairs | Pairs with different row labels |
| ---------------------- | --------------- | ---- | ----- | ------------------------------- |
| Shopify variant ID     | 0               | 0    | 0     | 0                               |
| SKU                    | 1               | 2    | 1     | 0                               |
| GTIN                   | 4               | 8    | 4     | 0                               |
| Shopify product ID     | 57              | 167  | 192   | 0                               |
| Product handle         | 57              | 167  | 192   | 0                               |
| Normalized title       | 57              | 167  | 192   | 0                               |
| Normalized description | 56              | 167  | 208   | 0                               |

Variant-ID equality would be the most direct catalog identity signal inside one retailer, but all variant IDs here are unique. SKU equality is useful when separate localized listings receive separate Shopify IDs. Require agreement on bundle contents, quantity, variant and condition before treating them as the same merchandise. Product ID and handle only identify families; titles and descriptions are often reused across variants.

## Duplicate groups and false positives

| Repeated SKU | Source records                                                                                                                        | Language interpretation                                      |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| `A8432022`   | [record 52](https://www.ankeritaly.com/products/a8433?variant=44489997090953), [record 66](https://www.ankeritaly.com/products/a8432) | Same-language listing candidates; no translation established |

Records 52 and 66 share SKU `A8432022` and GTIN `194644090876`, but both titles and descriptions are Italian. Their stored price amounts differ (`1599 EUR` versus `1199 EUR`), and their product and variant IDs differ. The broader length options in record 52 need checking before merging. Three other repeated GTINs each join a Nano charger to a charger-plus-earbuds bundle, so barcode agreement alone is unsafe.

## Detection procedure for this retailer

1. Extract row metadata, URL locale and every embedded content entry. Keep title language and description language separately; do not assign a whole record the title’s language.
2. Generate candidates within this retailer using exact variant ID first, then full SKU and corroborating GTIN. Keep product-ID, handle and model-family groups as weaker review queues. Exclude blank keys.
3. Detect language from substantive text after removing HTML, brand names and specification-only tokens. Mark short or empty text unknown. Inspect mismatches between detected field language and the declared row label.
4. Within identity candidates, compare actual field languages. Classify evidence as full translated content, title-only variation, mixed-language content, or same-language duplicate. Identical descriptions are not translated descriptions.
5. If identifiers differ, translation or multilingual semantic retrieval can propose candidates. Preserve and compare capacity, power, connectors, length, color, condition, quantity and bundle contents; reject conflicts before accepting identity. This semantic approach remains proposed, not measured here.
6. Preserve each listing’s URL, variant ID, price, currency, availability and scrape time. Link language versions to a common merchandise identity only when evidence supports it.

## What this snapshot can establish

The measured groups identify review candidates and concrete false positives, but no full different-language duplicate offer is established by this inspection. A single declared language does not prove that another locale is absent from the retailer’s live catalog; this snapshot only supports the findings above.
