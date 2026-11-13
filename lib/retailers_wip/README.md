# retailers_wip

Experimental retailer inventory modeled from the tables in `docs/retailer.md`.
The public `retailers` module provides `Retailer` and `SupportedLocales` entities.
Each brand module exposes `get_retailer(&str) -> Option<Retailer>` for exact
lowercase codes: 14 Anker, 10 Minisforum, and 21 Ugreen retailers (including NAS).

```rust
use retailers_wip::retailers::anker;

let retailer = anker::get_retailer("ankerau").unwrap();
assert_eq!(retailer.supported_locales[0].locale, "en-AU");
assert!(anker::get_retailer("unknown").is_none());
```

Factories contain static inventory definitions and return owned entities without
network or filesystem access. Unknown values and unselected collection countries
are `None`. Market codes, shipping destinations, and locale collection countries
are independent. Locale annotations preserve advertised-only and other evidence
qualifications; longer research prose stays in the source documentation.

Existing production integrations remain in their brand crates.
Run `make fmt` and `make ci` to validate this crate. Inventory tests compare all
factory records with the documentation tables.
