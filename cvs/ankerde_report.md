# ankerde same offer language detection

**One concrete title-language candidate was found:** SKU `A1638011` appears with German and English titles in records 137 and 142. Both rows are labeled `de` and share the same German description. This is partial localization of matching merchandise, not a fully translated duplicate offer. No pair with different declared row languages exists.

This report uses only [ankerde.csv](ankerde.csv), containing **196 records**. It asks whether that retailer lists the same merchandise in another language and which signals can detect it. Record numbers below are one-based data-row ordinals excluding the header, not physical CSV line numbers. Linked product URLs are snapshot evidence, not live availability checks.

## Language evidence

- Declared row languages: `{'de': 196}`.
- Embedded content languages: `{'de': 196}`; content-entry counts per row: `{1: 196}`.
- Embedded title or description differs from the row fields in **0 rows**. The content arrays therefore provide no extra translated entry.
- Missing descriptions: **9**. Empty text is unknown language, not evidence of translation.
- All URLs use `/de/products/`.

Language observations below come from inspecting titles and description excerpts, including repeated-SKU groups. No automated language classifier, machine translation or multilingual embedding model was run. Row labels and URL locale alone would miss mixed-language fields.

| Record and SKU                                                                                                          | Observed text language                                                                            |
| ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| [record 4](https://www.anker.com/de/products/anker-maggo-magnetic-wireless-charger-pad-15w-qi2-for-magsafe): `A25M0121` | English title and description despite `de`; description begins “Qi2 certified charging”.          |
| [record 11](https://www.anker.com/de/products/a3871011-82): `A3871011-82`                                               | English title and description despite `de`; accessory text begins “Authentic parts manufactured”. |
| [record 57](https://www.anker.com/de/products/a1256): `A1256G11`                                                        | German title with English description beginning “30 W maximum charging power”.                    |
| [record 165](https://www.anker.com/de/products/a210b-8-in-1-usb-c-hub-dual-4k-display-10gbps): `A210B0A1`               | English title and description beginning “Versatile 8-in-1 Connectivity”.                          |

## Measured matching approaches

A repeated-value group contains at least two records with the same nonempty key. “Rows” counts records in those groups; “pairs” counts unordered record pairs. Methods overlap. Identifier equality uses the stored value without trimming suffixes or GTIN padding. Normalized text strips HTML, decodes entities, collapses whitespace and case-folds.

| Key                    | Repeated groups | Rows | Pairs | Pairs with different row labels |
| ---------------------- | --------------- | ---- | ----- | ------------------------------- |
| Shopify variant ID     | 0               | 0    | 0     | 0                               |
| SKU                    | 13              | 27   | 15    | 0                               |
| GTIN                   | 0               | 0    | 0     | 0                               |
| Shopify product ID     | 45              | 131  | 174   | 0                               |
| Product handle         | 45              | 131  | 174   | 0                               |
| Normalized title       | 47              | 137  | 181   | 0                               |
| Normalized description | 42              | 140  | 363   | 0                               |

Variant-ID equality would be the most direct catalog identity signal inside one retailer, but all variant IDs here are unique. SKU equality is useful when separate localized listings receive separate Shopify IDs. Require agreement on bundle contents, quantity, variant and condition before treating them as the same merchandise. Product ID and handle only identify families; titles and descriptions are often reused across variants.

## Candidate missed by language metadata

**SKU `A1638011`: [record 137](https://www.anker.com/de/products/a1638-anker-nano-powerbank-mit-kabel-10000mah-45w?variant=59852733415505) and [record 142](https://www.anker.com/de/products/a1638).**

- German title: Anker Nano Powerbank (10K, 45W, Integriertes einziehbares USB-C Kabel).
- English title: Anker Nano Power Bank, 10,000 mAh External Battery with 70 cm Anker InstaCord Retractable Cable, 45 W Max Compact Battery, Smart Display, for iPhone 16 Series, iPad, Galaxy, Pixel and More.
- Descriptions are byte-for-byte identical German text; both row labels are `de`.
- Source IDs: `01a19361-e9d5-7282-a32a-e7946a9ed0fa` and `01a19361-ea0d-74e0-a5c2-1a4b8b000481`.
- Product IDs and variant IDs differ; both GTINs are missing. Stored price amount is `4999 EUR` in both rows, but availability differs: `available` versus `out_of_stock`. Both condition fields are empty.

The same full SKU and aligned capacity/power identify a strong same-merchandise candidate. The English title is visible evidence of title-level language variation. Different availability and separate catalog IDs mean these should remain separate listings linked to a common merchandise identity, not silently merged as identical commercial offers. A filter requiring different row language values would reject this useful pair.

## Duplicate groups and false positives

| Repeated SKU | Source records                                                                                                                                                                                                                                                                                                                                              | Language interpretation                                      |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| `A121D311`   | [record 62](https://www.anker.com/de/products/a121d-anker-nano-ladegerat-45w-smart-display-one-point-of-sale), [record 158](https://www.anker.com/de/products/a121d-anker-nano-ladegerat-45w-smart-display?variant=59852742492241), [record 168](https://www.anker.com/de/products/a121d-anker-nano-ladegerat-45w-smart-display-one?variant=59852742918225) | Same-language listing candidates; no translation established |
| `A17223Z1`   | [record 76](https://www.anker.com/de/products/a17223z1-c300), [record 195](https://www.anker.com/de/products/a17223z1-c300-one)                                                                                                                                                                                                                             | Same-language listing candidates; no translation established |
| `B25M1311`   | [record 95](https://www.anker.com/de/products/a25m1-maggo-qi2-kabellose-ladestation-magsafe-kompatibel-one?variant=59852724469841), [record 108](https://www.anker.com/de/products/a25m1-maggo-qi2-kabellose-ladestation-magsafe-kompatibel?variant=59852741148753)                                                                                         | Same-language listing candidates; no translation established |
| `A2544J21`   | [record 96](https://www.anker.com/de/products/a2544-maggo-qi2-ladegeraet-kabellos-magsafe-kompatibel-live?variant=59852728270929), [record 154](https://www.anker.com/de/products/a2544-maggo-qi2-ladegeraet-kabellos-magsafe-kompatibel?variant=59852749275217)                                                                                            | Same-language listing candidates; no translation established |
| `B25M8H21`   | [record 97](https://www.anker.com/de/products/a25m8-maggo-qi2-ladestation-kabellos-magsafe-kompatibel-live?variant=59852747309137), [record 110](https://www.anker.com/de/products/a25m8-maggo-qi2-ladestation-kabellos-magsafe-kompatibel?variant=59852736233553)                                                                                          | Same-language listing candidates; no translation established |
| `B25M8H11`   | [record 98](https://www.anker.com/de/products/a25m8-maggo-qi2-ladestation-kabellos-magsafe-kompatibel-live?variant=59852747341905), [record 111](https://www.anker.com/de/products/a25m8-maggo-qi2-ladestation-kabellos-magsafe-kompatibel?variant=59852736266321)                                                                                          | Same-language listing candidates; no translation established |
| `A2544J11`   | [record 101](https://www.anker.com/de/products/a2544-maggo-qi2-ladegeraet-kabellos-magsafe-kompatibel-live?variant=59852728303697), [record 155](https://www.anker.com/de/products/a2544-maggo-qi2-ladegeraet-kabellos-magsafe-kompatibel?variant=59852749307985)                                                                                           | Same-language listing candidates; no translation established |
| `B25M1321`   | [record 106](https://www.anker.com/de/products/a25m1-maggo-qi2-kabellose-ladestation-magsafe-kompatibel-one?variant=59852724437073), [record 107](https://www.anker.com/de/products/a25m1-maggo-qi2-kabellose-ladestation-magsafe-kompatibel?variant=59852741115985)                                                                                        | Same-language listing candidates; no translation established |
| `A110RH11`   | [record 121](https://www.anker.com/de/products/a110r-maggo-2-pro-qi2-magnetische-powerbank-cooling-25w?variant=60283769716817), [record 127](https://www.anker.com/de/products/a110r-99027000008)                                                                                                                                                           | Same-language listing candidates; no translation established |
| `A1638011`   | [record 137](https://www.anker.com/de/products/a1638-anker-nano-powerbank-mit-kabel-10000mah-45w?variant=59852733415505), [record 142](https://www.anker.com/de/products/a1638)                                                                                                                                                                             | German versus English titles; identical German description   |
| `A121D331`   | [record 159](https://www.anker.com/de/products/a121d-anker-nano-ladegerat-45w-smart-display?variant=59852742525009), [record 169](https://www.anker.com/de/products/a121d-anker-nano-ladegerat-45w-smart-display-one?variant=59852742950993)                                                                                                                | Same-language listing candidates; no translation established |
| `A121D3O1`   | [record 160](https://www.anker.com/de/products/a121d-anker-nano-ladegerat-45w-smart-display?variant=59852742426705), [record 166](https://www.anker.com/de/products/a121d-anker-nano-ladegerat-45w-smart-display-one?variant=59852742852689)                                                                                                                | Same-language listing candidates; no translation established |
| `A121D321`   | [record 161](https://www.anker.com/de/products/a121d-anker-nano-ladegerat-45w-smart-display?variant=59852742459473), [record 167](https://www.anker.com/de/products/a121d-anker-nano-ladegerat-45w-smart-display-one?variant=59852742885457)                                                                                                                | Same-language listing candidates; no translation established |

`A110RH11` occurs in records 121 and 127, but record 127 adds a magnetic metal phone holder in its title. Even identical SKU and shared text can conceal a bundle difference. Other repeated groups include point-of-sale and livestream listings. All GTINs are missing, so barcode matching is unavailable.

## Detection procedure for this retailer

1. Extract row metadata, URL locale and every embedded content entry. Keep title language and description language separately; do not assign a whole record the title’s language.
2. Generate candidates within this retailer using exact variant ID first, then full SKU and corroborating GTIN. Keep product-ID, handle and model-family groups as weaker review queues. Exclude blank keys.
3. Detect language from substantive text after removing HTML, brand names and specification-only tokens. Mark short or empty text unknown. Inspect mismatches between detected field language and the declared row label.
4. Within identity candidates, compare actual field languages. Classify evidence as full translated content, title-only variation, mixed-language content, or same-language duplicate. Identical descriptions are not translated descriptions.
5. If identifiers differ, translation or multilingual semantic retrieval can propose candidates. Preserve and compare capacity, power, connectors, length, color, condition, quantity and bundle contents; reject conflicts before accepting identity. This semantic approach remains proposed, not measured here.
6. Preserve each listing’s URL, variant ID, price, currency, availability and scrape time. Link language versions to a common merchandise identity only when evidence supports it.

## What this snapshot can establish

The actionable positive example is the German/English title pair for `A1638011`. It shows why per-field language detection must follow SKU grouping even when every row has the same language label. The file does not establish a fully translated German/English description pair for that SKU.
