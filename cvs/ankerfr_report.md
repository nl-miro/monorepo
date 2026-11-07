# ankerfr same offer language detection

**No same-offer pair in different languages was established within this file.** Every SKU, populated GTIN and variant ID is unique. Some individual rows contain English or German text despite `fr` metadata; this is evidence of incomplete localization, not a second-language duplicate.

This report uses only [ankerfr.csv](ankerfr.csv), containing **173 records**. It asks whether that retailer lists the same merchandise in another language and which signals can detect it. Record numbers below are one-based data-row ordinals excluding the header, not physical CSV line numbers. Linked product URLs are snapshot evidence, not live availability checks.

## Language evidence

- Declared row languages: `{'fr': 173}`.
- Embedded content languages: `{'fr': 173}`; content-entry counts per row: `{1: 173}`.
- Embedded title or description differs from the row fields in **0 rows**. The content arrays therefore provide no extra translated entry.
- Missing descriptions: **0**. Empty text is unknown language, not evidence of translation.
- All URLs use `/fr/products/`.

Language observations below come from inspecting titles and description excerpts, including repeated-SKU groups. No automated language classifier, machine translation or multilingual embedding model was run. Row labels and URL locale alone would miss mixed-language fields.

| Record and SKU                                                                                                                 | Observed text language                                                              |
| ------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| [record 28](https://www.anker.com/fr/products/a25x7-anker-prime-qi-2-2-charging-station-sans-fil-25w-3-en-1-stand): `B25X7G11` | French title with English description beginning “25W Ultra‑Fast Wireless Charging”. |
| [record 58](https://www.anker.com/fr/products/a8895): `A8895011`                                                               | French title with English description beginning “2-in-1 Charging Cable”.            |
| [record 62](https://www.anker.com/fr/products/a8195): `A81950Z1`                                                               | French title with German description beginning “DAS ANKER PLUS: Schließe dich”.     |
| [record 149](https://www.anker.com/fr/products/a210b-hub-anker-nano-85w-8-en-1): `A210B0A1`                                    | English title and description beginning “Versatile 8-in-1 Connectivity”.            |

## Measured matching approaches

A repeated-value group contains at least two records with the same nonempty key. “Rows” counts records in those groups; “pairs” counts unordered record pairs. Methods overlap. Identifier equality uses the stored value without trimming suffixes or GTIN padding. Normalized text strips HTML, decodes entities, collapses whitespace and case-folds.

| Key                    | Repeated groups | Rows | Pairs | Pairs with different row labels |
| ---------------------- | --------------- | ---- | ----- | ------------------------------- |
| Shopify variant ID     | 0               | 0    | 0     | 0                               |
| SKU                    | 0               | 0    | 0     | 0                               |
| GTIN                   | 0               | 0    | 0     | 0                               |
| Shopify product ID     | 43              | 121  | 158   | 0                               |
| Product handle         | 43              | 121  | 158   | 0                               |
| Normalized title       | 43              | 121  | 158   | 0                               |
| Normalized description | 41              | 122  | 252   | 0                               |

Variant-ID equality would be the most direct catalog identity signal inside one retailer, but all variant IDs here are unique. SKU equality is useful when separate localized listings receive separate Shopify IDs. Require agreement on bundle contents, quantity, variant and condition before treating them as the same merchandise. Product ID and handle only identify families; titles and descriptions are often reused across variants.

## Duplicate groups and false positives

No SKU is repeated. A same-offer translation would therefore need a different SKU or an identifier missing from this snapshot; product-family similarity alone cannot establish it.

For example, records 9 and 12 have different SKUs `A1695H11` and `A1695H41` under the same product. Their shared French title is product-level text, not two language versions. The foreign-language examples above have unique SKU and variant identifiers within this file.

## Detection procedure for this retailer

1. Extract row metadata, URL locale and every embedded content entry. Keep title language and description language separately; do not assign a whole record the title’s language.
2. Generate candidates within this retailer using exact variant ID first, then full SKU and corroborating GTIN. Keep product-ID, handle and model-family groups as weaker review queues. Exclude blank keys.
3. Detect language from substantive text after removing HTML, brand names and specification-only tokens. Mark short or empty text unknown. Inspect mismatches between detected field language and the declared row label.
4. Within identity candidates, compare actual field languages. Classify evidence as full translated content, title-only variation, mixed-language content, or same-language duplicate. Identical descriptions are not translated descriptions.
5. If identifiers differ, translation or multilingual semantic retrieval can propose candidates. Preserve and compare capacity, power, connectors, length, color, condition, quantity and bundle contents; reject conflicts before accepting identity. This semantic approach remains proposed, not measured here.
6. Preserve each listing’s URL, variant ID, price, currency, availability and scrape time. Link language versions to a common merchandise identity only when evidence supports it.

## What this snapshot can establish

The measured groups identify review candidates and concrete false positives, but no full different-language duplicate offer is established by this inspection. A single declared language does not prove that another locale is absent from the retailer’s live catalog; this snapshot only supports the findings above.
