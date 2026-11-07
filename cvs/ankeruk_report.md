# ankeruk offers in different languages within one retailer

This report examines only [ankeruk.csv](ankeruk.csv), containing **234 records**, to determine whether the same offer appears more than once in different languages. **No same-offer translation pair was established by the checks below.** Repeated titles, barcodes or product IDs alone do not establish translated copies.

## Language evidence

Row language counts: `{'en': 234}`. The embedded `content` arrays contain 234 entries with language counts `{'en': 234}`. Every row contains exactly one content entry. Embedded title and description equal their top-level fields in all 234 records when missing/null descriptions are treated as empty.

The snapshot scrape range is `2026-10-26 14:55:11.707719+00` to `2026-10-26 16:11:18.001308+00`. Recorded URLs below are lookup references, not live page checks. Record numbers are one-based data-record ordinals, excluding the CSV header.

All URLs use `/uk/`. Inspected titles and description excerpts are English. There are no differently labeled language pairs and no alternate-language entries inside `content`. This establishes that the supplied export has no explicit translation pairing; it does not establish what other languages the live retailer may offer.

## Measured matching approaches

A repeated group has at least two records with the same nonempty key. Rows counts records inside those groups; pairs counts unordered pairs, once each. The final column counts only pairs whose declared row languages differ. Methods overlap and their totals must not be added.

| Approach                                        | Repeated groups | Rows | Pairs | Different-label pairs |
| ----------------------------------------------- | --------------- | ---- | ----- | --------------------- |
| Exact Shopify variant ID                        | 0               | 0    | 0     | 0                     |
| Exact SKU                                       | 0               | 0    | 0     | 0                     |
| Exact GTIN                                      | 0               | 0    | 0     | 0                     |
| Locale-stripped URL with variant query retained | 0               | 0    | 0     | 0                     |
| Exact Shopify product ID                        | 43              | 167  | 451   | 0                     |
| Exact product handle                            | 43              | 167  | 451   | 0                     |
| Five-character SKU prefix                       | 45              | 157  | 247   | 0                     |
| Normalized exact title                          | 45              | 173  | 457   | 0                     |
| Normalized exact description                    | 49              | 186  | 521   | 0                     |

Exact identifier comparison preserves the complete SKU, including suffixes and bundle composition. Empty GTINs do not match. URL normalization removes only the observed locale segment, retains the entire query including `variant`, and drops fragments. Text normalization strips HTML, decodes entities, collapses whitespace and case-folds. The five-character prefix is an exploratory family grouping for A/B/T/E model-like SKUs; bundle and combo SKUs are excluded.

## Concrete findings

- [record 2: soundcore 3 \| Portable Bluetooth Speaker with Stereo Sound, IPX7 Waterproof](https://www.anker.com/uk/products/a3117011?variant=47420259729583) (`A3117011`), [record 3: soundcore 3 \| Portable Bluetooth Speaker with Stereo Sound, IPX7 Waterproof](https://www.anker.com/uk/products/a3117011?variant=47420259762351) (`A3117031`), [record 4: soundcore 3 \| Portable Bluetooth Speaker with Stereo Sound, IPX7 Waterproof](https://www.anker.com/uk/products/a3117011?variant=47420259795119) (`A3117091`) and [record 5: soundcore 3 \| Portable Bluetooth Speaker with Stereo Sound, IPX7 Waterproof](https://www.anker.com/uk/products/a3117011?variant=47420259827887) (`A31170A1`) have the same English soundcore 3 title, but different SKUs. They illustrate variant grouping rather than language duplication.
- [record 1: Q20i Headphones - Blue (Refurbished)](https://www.anker.com/uk/products/a3004g31-f0) (`A3004G31-F0`) explicitly identifies a refurbished product. Condition and suffixes must remain part of identity checks when looking for translated equivalents.
- All 234 SKUs and Shopify variant IDs are unique; every GTIN is missing. Barcode matching is therefore unavailable. The 43 repeated product-ID groups contain 167 records, and must not be collapsed into 43 offers.

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
