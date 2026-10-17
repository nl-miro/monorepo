# development

Developer tools and a server-rendered database dashboard at `/dashboard`. The
home page links to the dashboard. Reporting clients and response structs live
inside this app's server binary; service credentials never reach the browser.

## Run

```sh
PORT=3005 make serve
```

Open `http://localhost:3005/dashboard`. Each page load fetches both reports
concurrently, with a five-second timeout per service. Use **Refresh reports** to
reload; there is no background polling. One service can fail without hiding the
other's report.

| Environment variable | Default |
| --- | --- |
| `RETAILER_SOURCING_URL` | `http://172.28.100.40:3000` |
| `RETAILER_SOURCING_TOKEN` | Repository development bearer token for retailer sourcing |
| `OBJECT_STORAGE_URL` | `http://172.28.100.43:3000` |
| `OBJECT_STORAGE_TOKEN` | Repository development bearer token for object storage |
| `PORT` | `3000` |

Export overrides in the shell before starting the app. The existing CLI tools
retain their own defaults. The dashboard uses the app's existing access model
and is intended for the development environment.

## Reading the report

Each service reports ordinary tables in its connection's active PostgreSQL
schema, including migration and message bus tables. Row counts are **estimates**
from PostgreSQL statistics and can lag recent writes. A zero estimate does not
prove a table is empty. Reporting neither counts table rows nor runs ANALYZE.

Table bytes include TOAST and auxiliary storage. Index bytes cover indexes
attached to the table; total bytes are their sum. The dashboard displays IEC
units, with exact byte values available on hover and to assistive technology.
Tables are sorted by total size. Combined totals cover only available services.

## Testing

```sh
make fmt
make ci
```

Rust tests cover private HTTP clients, authentication, timeouts, malformed
responses, partial failure, HTML escaping, sorting, and page routes. The CSS in
`src/dashboard/style.css` is embedded into the HTML; no JavaScript or frontend
build is required. After CSS changes, run the app and check `/dashboard` in a
browser at desktop and narrow mobile widths. Verify that the page fits the
viewport, wide tables scroll within their cards, and Refresh remains usable.
Also stop one upstream service and check that the other report remains visible.
