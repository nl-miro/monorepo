# ankercom offers in different languages within one retailer

This report examines only [ankercom.csv](ankercom.csv), containing **250 records**, to determine whether the same offer appears more than once in different languages. **No same-offer translation pair was established by the checks below.** Repeated titles, barcodes or product IDs alone do not establish translated copies.

## Language evidence

Row language counts: `{'en': 250}`. The embedded `content` arrays contain 250 entries with language counts `{'en': 250}`. Every row contains exactly one content entry. Embedded title and description equal their top-level fields in all 250 records when missing/null descriptions are treated as empty.

The snapshot scrape range is `2026-10-26 14:55:39.289323+00` to `2026-10-26 16:32:34.853622+00`. Recorded URLs below are lookup references, not live page checks. Record numbers are one-based data-record ordinals, excluding the CSV header.

All URLs use `/products/`, without a language prefix. Inspected titles and description excerpts are English. There are no differently labeled language pairs and no alternate-language entries inside `content`. This establishes that the supplied export has no explicit translation pairing; it does not establish what other languages the live retailer may offer.

## Measured matching approaches

A repeated group has at least two records with the same nonempty key. Rows counts records inside those groups; pairs counts unordered pairs, once each. The final column counts only pairs whose declared row languages differ. Methods overlap and their totals must not be added.

| Approach                                        | Repeated groups | Rows | Pairs | Different-label pairs |
| ----------------------------------------------- | --------------- | ---- | ----- | --------------------- |
| Exact Shopify variant ID                        | 0               | 0    | 0     | 0                     |
| Exact SKU                                       | 3               | 6    | 3     | 0                     |
| Exact GTIN                                      | 44              | 100  | 76    | 0                     |
| Locale-stripped URL with variant query retained | 0               | 0    | 0     | 0                     |
| Exact Shopify product ID                        | 48              | 198  | 558   | 0                     |
| Exact product handle                            | 48              | 198  | 558   | 0                     |
| Five-character SKU prefix                       | 42              | 153  | 307   | 0                     |
| Normalized exact title                          | 48              | 198  | 558   | 0                     |
| Normalized exact description                    | 46              | 200  | 588   | 0                     |

Exact identifier comparison preserves the complete SKU, including suffixes and bundle composition. Empty GTINs do not match. URL normalization removes only the observed locale segment, retains the entire query including `variant`, and drops fragments. Text normalization strips HTML, decodes entities, collapses whitespace and case-folds. The five-character prefix is an exploratory family grouping for A/B/T/E model-like SKUs; bundle and combo SKUs are excluded.

## Concrete findings

- [record 69: Anker Laptop Power Bank (25K, 165W, Built-In and Retractable Cables)](https://www.anker.com/products/a1695-anker-power-bank-25000mah-165w?variant=46386686394518) (`BUNDLE-B26971Z1-1-A1695H11-1`) and [record 151: Bundle: Anker Laptop Power Bank and Anker Laptop Charger](https://www.anker.com/products/a1695-a2697?variant=44350422843542) (`BUNDLE-B26971Z1-1-A1695H11-1`) share full bundle SKU `BUNDLE-B26971Z1-1-A1695H11-1`. Both titles and descriptions are English. The first title omits the charger named in the second; this is an alternative bundle listing, not evidence of another language.
- [record 21: Refurbished Anker Nano Power Bank (22.5W, Built-In USB-C Connector)](https://www.anker.com/products/a1653-f0?variant=43907361308822) (`A1653011-F0`) and [record 182: Anker Nano Power Bank (22.5W, Built-In USB-C Connector)](https://www.anker.com/products/a1653-usb-c-portable-charger-5000mah?variant=42733234258070) (`A1653011`) share GTIN `194644145880`, but the first is refurbished and has SKU suffix `-F0`. Barcode equality must not erase the condition difference.
- [record 201: Anker MagGo Wireless Charging Station (3-in-1, Foldable Pad, Aurora White, with Travel Case & Adapter)](https://www.anker.com/products/a25m8j21-1) (`B25M8J21-1`) and [record 247: Anker MagGo Wireless Charging Station (3-in-1, Foldable Pad)](https://www.anker.com/products/a25m8-maggo-qi2-wireless-charging-station-magsafe-compatible?variant=46418088919190) (`B25M8J21-1`) share SKU `B25M8J21-1` and exactly the same description. The title varies in included detail, but both are English.
- All 250 variant IDs are unique. Three repeated-SKU groups contain six rows; 17 GTINs are missing.

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
