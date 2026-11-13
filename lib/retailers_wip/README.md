# retailers_wip

Experimental retailer inventory modeled from the tables in `docs/retailer.md`.
The public `retailers` module provides `Retailer` and `SupportedLocales` entities,
plus `Market`, `Country`, `Region`, `Locale`, and `Language` enums.
`Retailer.retailer_code` uses the shared `RetailerCode` enum, re-exported here.
Each brand module exposes `get_retailer(&str) -> Option<Retailer>` for exact
lowercase codes: 19 Anker, 10 Minisforum, and 21 Ugreen retailers (including NAS).

```rust
use retailers_wip::retailers::{anker, Locale, RetailerCode};

let retailer = anker::get_retailer("ankerau").unwrap();
assert_eq!(retailer.retailer_code, RetailerCode::AnkerAu);
assert_eq!(retailer.supported_locales[0].locale, Locale::en_AU);
assert!(anker::get_retailer("unknown").is_none());
```

Factories contain static inventory definitions and return owned entities without
network or filesystem access. Unknown target markets and shipping countries use
empty vectors of `Market`. Locale `country_region` is `Option<Market>`.
`Market::Country(Country::Austria)` and
`Market::Region(Region::EuropeanUnion)` distinguish countries from regions.
`as_str()` returns inventory codes such as `AT` and `EU`; `from_code()` accepts
exact uppercase codes represented in the inventory and returns `None` otherwise.
Other unknown optional values and unselected collection countries
are `None`. Market codes, shipping destinations, and locale collection countries
are independent. Only main-table locales appear in `supported_locales`; advertised-only entries
remain in the documentation. Countryless main-table entries remain supported.
Use `retailers::get_retailer(RetailerCode)` to load an entity and
`Retailer::supported_locales()` to access its collection locales. Longer research
prose stays in the source documentation.

Existing production integrations remain in their brand crates.
Run `make fmt` and `make ci` to validate this crate. Inventory tests compare all
factory records with the documentation tables.

The Anker inventory includes five separate Nordic market codes (`ankerse`,
`ankerno`, `ankerfi`, `ankeris`, `ankerdk`) and the historical combined
`ankernordicscom` entry. All share the Nordic storefront and backend.

Shipping-configuration survey results remain available as research evidence in
[`retailer-shipping-countries-2026-11-13.json`](../../docs/research/retailer-shipping-countries-2026-11-13.json).
They are not part of the retailer entity.

`SupportedLocales.locale` uses the `Locale` enum for all 106 distinct inventory
locales. Variants use locale notation, such as `Locale::en_CA` and `Locale::en`.
`Locale::as_str()` returns canonical tags such as `en-DK`, and
`Locale::from_code()` accepts case-insensitive known tags (including `en-dk`).
Unknown tags, whitespace and underscore separators return `None`. Source
spellings remain in the documentation; locale tags do not determine the
independent GraphQL country or language fields.

`SupportedLocales.language` uses `Language` variants such as `Language::EN` and
`Language::ZH_TW`. `as_str()` returns the exact GraphQL code; `from_code()`
accepts only uppercase codes represented in the inventory. The definitions live in
[`language.rs`](src/retailers/language.rs), based on the GraphQL language column in
[the inventory](../../docs/retailer.md).

`Retailer::initial_fill_contexts()` snapshots main-table locales as unique optional
country/language requests, preserving inventory order. It rejects empty lists and
regional country values. New requests have no expected currency; historical queued
currency expectations remain readable. `initial_fill` also exposes the local
integration extension used by the brand engines for these requests, while stored
artifacts keep the published, fully observed market-context format.

The `graphql_errors` module formats upstream GraphQL diagnostics for all three
brand engines before typed data decoding. Errors retain the operation name,
messages, paths, source locations, and diagnostic extensions (code, required
access, documentation and query-cost/throttle information). Missing/null data
without errors has a separate message. Request headers, product data and arbitrary
extensions are excluded; engines redact the active access token if it is echoed.
For example, a denied selling-plan query reports `OfferDetails`, `ACCESS_DENIED`,
`product.sellingPlanGroups` (as a JSON path array), and the
`unauthenticated_read_selling_plans` requirement through the existing queue error.
Retry policy and the queue's `storage error` prefix remain unchanged.
