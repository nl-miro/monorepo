# ankereu same offer language detection

**No different-language version was identified within this file.** All language metadata and URL locales are English, and inspected titles and description excerpts are English. Repeated SKUs identify alternative English listings; GTIN reuse also joins bundles to standalone products.

This report uses only [ankereu.csv](ankereu.csv), containing **224 records**. It asks whether that retailer lists the same merchandise in another language and which signals can detect it. Record numbers below are one-based data-row ordinals excluding the header, not physical CSV line numbers. Linked product URLs are snapshot evidence, not live availability checks.

## Language evidence

- Declared row languages: `{'en': 224}`.
- Embedded content languages: `{'en': 224}`; content-entry counts per row: `{1: 224}`.
- Embedded title or description differs from the row fields in **0 rows**. The content arrays therefore provide no extra translated entry.
- Missing descriptions: **0**. Empty text is unknown language, not evidence of translation.
- All URLs use `/eu-en/products/`.

Language observations below come from inspecting titles and description excerpts, including repeated-SKU groups. No automated language classifier, machine translation or multilingual embedding model was run. Row labels and URL locale alone would miss mixed-language fields.

## Measured matching approaches

A repeated-value group contains at least two records with the same nonempty key. “Rows” counts records in those groups; “pairs” counts unordered record pairs. Methods overlap. Identifier equality uses the stored value without trimming suffixes or GTIN padding. Normalized text strips HTML, decodes entities, collapses whitespace and case-folds.

| Key                    | Repeated groups | Rows | Pairs | Pairs with different row labels |
| ---------------------- | --------------- | ---- | ----- | ------------------------------- |
| Shopify variant ID     | 0               | 0    | 0     | 0                               |
| SKU                    | 16              | 32   | 16    | 0                               |
| GTIN                   | 8               | 44   | 156   | 0                               |
| Shopify product ID     | 54              | 168  | 383   | 0                               |
| Product handle         | 54              | 168  | 383   | 0                               |
| Normalized title       | 55              | 172  | 391   | 0                               |
| Normalized description | 52              | 172  | 915   | 0                               |

Variant-ID equality would be the most direct catalog identity signal inside one retailer, but all variant IDs here are unique. SKU equality is useful when separate localized listings receive separate Shopify IDs. Require agreement on bundle contents, quantity, variant and condition before treating them as the same merchandise. Product ID and handle only identify families; titles and descriptions are often reused across variants.

## Duplicate groups and false positives

| Repeated SKU                   | Source records                                                                                                                                                                                                                 | Language interpretation                                      |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------ |
| `BUNDLE-A121D311-1-A82E2011-1` | [record 20](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-with-a82e2-cable?variant=55534053491064), [record 170](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886229368) | Same-language listing candidates; no translation established |
| `BUNDLE-A121D321-1-A82E2021-1` | [record 50](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-with-a82e2-cable?variant=55534053458296), [record 169](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886196600) | Same-language listing candidates; no translation established |
| `BUNDLE-A121D331-1-A82E2011-1` | [record 58](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-with-a82e2-cable?variant=55534053523832), [record 157](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886262136) | Same-language listing candidates; no translation established |
| `BUNDLE-A121D321-1-A82E2022-1` | [record 59](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-with-a82e2-cable?variant=55534053556600), [record 158](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886294904) | Same-language listing candidates; no translation established |
| `BUNDLE-A121D311-1-A82E2012-1` | [record 60](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-with-a82e2-cable?variant=55534053589368), [record 159](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886327672) | Same-language listing candidates; no translation established |
| `BUNDLE-A121D331-1-A82E2012-1` | [record 61](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-with-a82e2-cable?variant=55534053622136), [record 160](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886360440) | Same-language listing candidates; no translation established |
| `BUNDLE-A121D311-2`            | [record 114](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532885901688), [record 138](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049034616)          | Same-language listing candidates; no translation established |
| `BUNDLE-A121D3O1-1-A121D311-1` | [record 125](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049296760), [record 168](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886163832)          | Same-language listing candidates; no translation established |
| `BUNDLE-A121D321-2`            | [record 132](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532885868920), [record 134](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049001848)          | Same-language listing candidates; no translation established |
| `BUNDLE-A121D311-1-A121D331-1` | [record 135](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049198456), [record 154](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886065528)          | Same-language listing candidates; no translation established |
| `BUNDLE-A121D331-2`            | [record 139](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049067384), [record 149](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532885934456)          | Same-language listing candidates; no translation established |
| `BUNDLE-A121D3O1-2`            | [record 140](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049100152), [record 150](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532885967224)          | Same-language listing candidates; no translation established |
| `BUNDLE-A121D311-1-A121D321-1` | [record 141](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049132920), [record 166](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532885999992)          | Same-language listing candidates; no translation established |
| `BUNDLE-A121D331-1-A121D321-1` | [record 142](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049165688), [record 167](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886032760)          | Same-language listing candidates; no translation established |
| `BUNDLE-A121D3O1-1-A121D331-1` | [record 143](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049231224), [record 155](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886098296)          | Same-language listing candidates; no translation established |
| `BUNDLE-A121D3O1-1-A121D321-1` | [record 144](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger-2-pack?variant=55534049263992), [record 156](https://www.anker.com/eu-en/products/a121d-45w-usb-c-fast-charger?variant=55532886131064)          | Same-language listing candidates; no translation established |

GTIN `194644203504` joins record 5 (`BUNDLE-A1664H21-1-A2637327-1`) to record 115 (`A1664H21`): the first includes a charger, the second is the standalone power bank. GTIN-only grouping would create a false same-offer match. SKU `BUNDLE-A121D311-2` appears in records 114 and 138 under a generic charger title and a “(2-Pack)” title; both are English and their availability differs.

## Detection procedure for this retailer

1. Extract row metadata, URL locale and every embedded content entry. Keep title language and description language separately; do not assign a whole record the title’s language.
2. Generate candidates within this retailer using exact variant ID first, then full SKU and corroborating GTIN. Keep product-ID, handle and model-family groups as weaker review queues. Exclude blank keys.
3. Detect language from substantive text after removing HTML, brand names and specification-only tokens. Mark short or empty text unknown. Inspect mismatches between detected field language and the declared row label.
4. Within identity candidates, compare actual field languages. Classify evidence as full translated content, title-only variation, mixed-language content, or same-language duplicate. Identical descriptions are not translated descriptions.
5. If identifiers differ, translation or multilingual semantic retrieval can propose candidates. Preserve and compare capacity, power, connectors, length, color, condition, quantity and bundle contents; reject conflicts before accepting identity. This semantic approach remains proposed, not measured here.
6. Preserve each listing’s URL, variant ID, price, currency, availability and scrape time. Link language versions to a common merchandise identity only when evidence supports it.

## What this snapshot can establish

The measured groups identify review candidates and concrete false positives, but no full different-language duplicate offer is established by this inspection. A single declared language does not prove that another locale is absent from the retailer’s live catalog; this snapshot only supports the findings above.
