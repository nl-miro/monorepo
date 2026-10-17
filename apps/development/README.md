# development

Developer tools for processing saved retailer sitemaps and offer pages, plus a
Poem server with a home page at `/`.

## Offline tools

Run these targets from the repository root. `DATA_DIR` defaults to `../data`;
`RETAILERS="ankerau ankerjapancom"` limits the offer stages to those storefronts.

| Target | Input and output |
| --- | --- |
| `process-sitemap-sources` | `sitemaps/sources/<retrieval id>/` to `sitemaps/processed/<retrieval id>.json` |
| `consolidate-sitemap-sources` | Processed sitemap trees to `sitemaps/consolidate/` URL groups |
| `extract-offer-details` | `offer_details/sources/*.html` to `offer_details/extracted/*.json` |
| `normalize-offer-details` | Extracted offers to `offer_details/normalized/` |
| `structuralize-offer-details` | Normalized offers to `offer_details/structuralized/` |
| `process-offer-details` | All offer stages through the extractor crate's CLI; its final folder is `structurized/` |

Paths are under `<DATA_DIR>/<retailer>/`. Failed offer stages remove stale output
and report errors while continuing with other pages.

`dump-sitemap-sources` and `dump-offer-details` download source files from the
retailer sourcing development API. They require that API to be available; set
`RETAILER_SOURCING_URL` (default `http://localhost:3000`) and optionally
`RETAILER_SOURCING_TOKEN` to override the repository development token.
The offline stages operate on existing files without a running service.

## Run and test

```sh
make fmt
make ci
PORT=3005 make serve
curl http://localhost:3005/
```

Tests cover the home page and the tools' file handling, stage failures, stale
output cleanup, and sitemap processing.
