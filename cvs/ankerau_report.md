# ankerau offers in different languages within one retailer

This report examines only [ankerau.csv](ankerau.csv), containing **241 records**, to determine whether the same offer appears more than once in different languages. **No same-offer translation pair was established by the checks below.** Repeated titles, barcodes or product IDs alone do not establish translated copies.

## Language evidence

Row language counts: `{'en': 241}`. The embedded `content` arrays contain 241 entries with language counts `{'en': 241}`. Every row contains exactly one content entry. Embedded title and description equal their top-level fields in all 241 records when missing/null descriptions are treated as empty.

The snapshot scrape range is `2026-10-26 14:55:20.18409+00` to `2026-10-26 16:34:54.711479+00`. Recorded URLs below are lookup references, not live page checks. Record numbers are one-based data-record ordinals, excluding the CSV header.

All URLs use `/au/`. Inspected titles and description excerpts are English. There are no differently labeled language pairs and no alternate-language entries inside `content`. This establishes that the supplied export has no explicit translation pairing; it does not establish what other languages the live retailer may offer.

## Measured matching approaches

A repeated group has at least two records with the same nonempty key. Rows counts records inside those groups; pairs counts unordered pairs, once each. The final column counts only pairs whose declared row languages differ. Methods overlap and their totals must not be added.

| Approach                                        | Repeated groups | Rows | Pairs | Different-label pairs |
| ----------------------------------------------- | --------------- | ---- | ----- | --------------------- |
| Exact Shopify variant ID                        | 0               | 0    | 0     | 0                     |
| Exact SKU                                       | 0               | 0    | 0     | 0                     |
| Exact GTIN                                      | 5               | 11   | 7     | 0                     |
| Locale-stripped URL with variant query retained | 0               | 0    | 0     | 0                     |
| Exact Shopify product ID                        | 52              | 183  | 399   | 0                     |
| Exact product handle                            | 52              | 183  | 399   | 0                     |
| Five-character SKU prefix                       | 57              | 172  | 242   | 0                     |
| Normalized exact title                          | 52              | 185  | 405   | 0                     |
| Normalized exact description                    | 53              | 186  | 405   | 0                     |

Exact identifier comparison preserves the complete SKU, including suffixes and bundle composition. Empty GTINs do not match. URL normalization removes only the observed locale segment, retains the entire query including `variant`, and drops fragments. Text normalization strips HTML, decodes entities, collapses whitespace and case-folds. The five-character prefix is an exploratory family grouping for A/B/T/E model-like SKUs; bundle and combo SKUs are excluded.

## Concrete findings

- GTIN `194644203467` occurs in [record 115: Anker MagGo Power Bank (Ultra-Slim, 10K, MagSafe)](https://www.anker.com/au/products/a1664-maggo-10000mah-power-bank?variant=50466994979119) (`A1664H11`), [record 140: Anker MagGo Power Bank (Ultra-Slim, 10K, MagSafe)](https://www.anker.com/au/products/bundle-a1664h11-1) (`BUNDLE-A1664H11-1`) and [record 142: 2 × Anker MagGo Power Bank, Ultra-Slim 10,000mAh MagSafe](https://www.anker.com/au/products/bundle-a1664h11-1-a1664h21-1) (`BUNDLE-A1664H11-1-A1664H21-1`). The last listing contains two power banks; it is not a translation of the standalone listing. The first two have the same English title, not two languages.
- [record 191: Anker Nano Power Bank (30W, Built-In USB-C Cable)](https://www.anker.com/au/products/a1259-built-in-cable-power-bank-10000mah?variant=49565152084271) (`A1259H11`) and [record 194: Anker Nano Power Bank (30W, Built-In USB-C Cable)](https://www.anker.com/au/products/a1259-built-in-cable-power-bank-10000mah?variant=47837927473455) (`A12590V1`) share GTIN `194644148652` and an English title but have different full SKUs. Treat them as variant or source-data candidates, not translated duplicates.
- All 241 SKUs and variant IDs are unique. There are 33 missing GTINs. Same-product grouping primarily collects variants: 52 repeated product-ID groups include 183 rows.

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
