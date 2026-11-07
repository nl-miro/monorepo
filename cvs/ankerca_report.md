# ankerca offers in different languages within one retailer

This report examines only [ankerca.csv](ankerca.csv), containing **167 records**, to determine whether the same offer appears more than once in different languages. **No same-offer translation pair was established by the checks below.** Repeated titles, barcodes or product IDs alone do not establish translated copies.

## Language evidence

Row language counts: `{'fr': 160, 'en': 7}`. The embedded `content` arrays contain 167 entries with language counts `{'fr': 160, 'en': 7}`. Every row contains exactly one content entry. Embedded title and description equal their top-level fields in all 167 records when missing/null descriptions are treated as empty.

The snapshot scrape range is `2026-10-26 14:55:10.384366+00` to `2026-10-26 16:33:33.022886+00`. Recorded URLs below are lookup references, not live page checks. Record numbers are one-based data-record ordinals, excluding the CSV header.

The 160 French-labeled rows use `/ca-fr/`; the seven English-labeled rows use `/ca/`. This is the only multilingual population in this file, but different locales alone do not identify equivalent offers. There are 1,120 possible English–French row pairs. None shares an exact SKU, GTIN, Shopify variant ID, product ID, handle or locale-stripped URL.

French metadata is not a reliable language classifier: records 55 and 137 have French titles but identical English descriptions beginning “Small Yet Powerful”. Records 71–73 and 112 have English titles despite their `fr` labels. All ten repeated-SKU groups were inspected: their titles are French on both sides and each group has one identical description value. They do not supply an English/French version pair.

## Measured matching approaches

A repeated group has at least two records with the same nonempty key. Rows counts records inside those groups; pairs counts unordered pairs, once each. The final column counts only pairs whose declared row languages differ. Methods overlap and their totals must not be added.

| Approach                                        | Repeated groups | Rows | Pairs | Different-label pairs |
| ----------------------------------------------- | --------------- | ---- | ----- | --------------------- |
| Exact Shopify variant ID                        | 0               | 0    | 0     | 0                     |
| Exact SKU                                       | 10              | 24   | 18    | 0                     |
| Exact GTIN                                      | 0               | 0    | 0     | 0                     |
| Locale-stripped URL with variant query retained | 0               | 0    | 0     | 0                     |
| Exact Shopify product ID                        | 38              | 95   | 82    | 0                     |
| Exact product handle                            | 38              | 95   | 82    | 0                     |
| Five-character SKU prefix                       | 29              | 84   | 130   | 1                     |
| Normalized exact title                          | 38              | 97   | 99    | 0                     |
| Normalized exact description                    | 35              | 101  | 147   | 0                     |

Exact identifier comparison preserves the complete SKU, including suffixes and bundle composition. Empty GTINs do not match. URL normalization removes only the observed locale segment, retains the entire query including `variant`, and drops fragments. Text normalization strips HTML, decodes entities, collapses whitespace and case-folds. The five-character prefix is an exploratory family grouping for A/B/T/E model-like SKUs; bundle and combo SKUs are excluded.

## Concrete findings

All seven English-labeled records were checked against the French-labeled population. The model-prefix comparison produces only one candidate:

| Record | SKU           | English title                                                  | Within-file finding                                                         |
| ------ | ------------- | -------------------------------------------------------------- | --------------------------------------------------------------------------- |
| 2      | T2080111-91   | Power Cord US For S1 Pro                                       | No French-labeled record shares the five-character SKU prefix.              |
| 3      | T211A110-87SR | station(CA)                                                    | Prefix matches record 16, but base station and dustbin are different parts. |
| 6      | A3134012      | Boom 2 Plus \| Powerful Outdoor Bass Bluetooth Speaker         | No French-labeled record shares the five-character SKU prefix.              |
| 7      | A3134062      | Boom 2 Plus \| Powerful Outdoor Bass Bluetooth Speaker         | No French-labeled record shares the five-character SKU prefix.              |
| 8      | A3134032      | Boom 2 Plus \| Powerful Outdoor Bass Bluetooth Speaker         | No French-labeled record shares the five-character SKU prefix.              |
| 9      | E81701WE      | eufy LocalSecure System (Premium Solution with Video Doorbell) | No French-labeled record shares the five-character SKU prefix.              |
| 10     | A3004Z11-F0   | Refurbished Q20i \| Hybrid Active Noise Cancelling Headphones  | No French-labeled record shares the five-character SKU prefix.              |

- [record 3: station(CA)](https://www.anker.com/ca/products/t211a110-87sr) (`T211A110-87SR`) is an English base station, while [record 16: Poubelle](https://www.anker.com/ca-fr/products/t211a110-81sr) (`T211A110-81SR`) is a French dustbin (`Poubelle`). Both share prefix `T211A`, but their full suffixes `-87SR` and `-81SR` and descriptions identify different replacement parts. Reject this pair.
- [record 55: Chargeur Anker 511 (Nano 3, 30 W) - Cadeau](https://www.anker.com/ca-fr/products/a2147-gift) (`A2147113`) and [record 137: Chargeur Anker 511 (Nano 3, 30 W)](https://www.anker.com/ca-fr/products/a2147?variant=47654624788669) (`A2147113`) share SKU `A2147113`, French titles and an identical English description. The first is a gift listing, with raw price amount `2499 CAD` versus `2999 CAD`. They are not an English/French pair.
- [record 31: Batterie externe Anker (25 000 mAh, 165 W, câbles intégrés et rétractables) Argent](https://www.anker.com/ca-fr/products/a1695-silver) (`A1695H41`) and [record 144: Batterie externe Anker (25 K, 165 W, câbles intégrés et rétractables)](https://www.anker.com/ca-fr/products/a1695-anker-power-bank-25000mah-165w?variant=47654623903933) (`A1695H41`) share SKU `A1695H41`, French descriptions, and French titles with different wording. This is another same-language candidate, not a translation.
- All 167 GTINs are missing and every variant ID is unique. Ten repeated-SKU groups cover 24 rows. All have identical descriptions within each group.

## Detection approaches to use within this retailer

| Approach                                       | How to use it                                                                                                               | Failure to avoid                                                    |
| ---------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| Variant identity plus language evidence        | Group the same variant ID or locale-normalized variant URL, then compare title and description languages.                   | One product ID can include many distinct variants.                  |
| Full SKU with corroborating attributes         | Match full SKU; check GTIN when present, condition, quantity, included accessories and specifications.                      | Bundles, gifts and refurbished listings can reuse identifiers.      |
| Field-level language assessment                | Assess title and description separately, including content entries; review mixed or short text.                             | A locale or language label can accompany untranslated English copy. |
| Translation or multilingual semantic retrieval | Retrieve same-retailer candidates without identical IDs, then validate numeric specifications and exact variant attributes. | Similar product-family wording is not proof of the same offer.      |

The identifier, URL and exact-text counts above were executed. Field-language observations come from inspection of the cited examples and candidate groups; no full-corpus language classifier, machine translation or multilingual embedding run is claimed. Semantic retrieval remains a proposed fallback. Exact text equality detects reused wording, not translations.

A practical detector should require two independent findings: the records identify the same sellable variant or package, and at least one substantive field is written in a different language. Mark mixed-language content separately. Keep each listing URL, price, availability and scrape time even when product identity matches. Prices can differ between localizations and should not be the identity key.

For this snapshot, retain the identified same-language duplicates and family candidates for data-quality review, but do not label them as translated offers. To evaluate true-positive detection later, obtain another locale export from this same retailer with overlapping variants and verify a small labeled set before choosing thresholds.
