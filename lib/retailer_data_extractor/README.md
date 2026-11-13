# retailer_data_extractor

Facade for retailer extraction, sitemap discovery and initial fills.

Anker, Minisforum and Ugreen run from local crates. Mi remains disconnected.
All supported Anker GraphQL storefronts use the shared complete extracted model
in `retailers_anker::extracted`. It preserves every modeled raw field using the
AU/CA-style grouping. Minisforum and Ugreen continue typed raw v2 parsing.
Localization and JSON pretty printing remain available.

Run `make fmt` and `make ci` here to verify the facade.

See the [Anker README](../retailers_anker/README.md#shared-complete-extracted-model) for the output contract and validation.
