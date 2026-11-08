# ADR 48: Multi-retailer fixed GraphQL operations

Status: Accepted

Extends [ADR 47](0047-fixed-graphql-operations.md).

## Context

The fixed catalog offers and offer details comparison now covers Anker AU and
Anker Canada. A hardcoded AU endpoint, context and token would silently send
Canadian requests to the wrong store. Callers need to select the storefront,
and saved artifacts must remain separated without run IDs or extra requests.

## Decision

Add `Storefront::{AnkerAu, AnkerCa}` as the first argument to each of the six
public operation functions. The enum holds retailer configuration: slug,
endpoint, country, language, currency and public browser Storefront token.
Each operation appends its retailer slug to the caller's output root once.

Share configuration and JSON persistence, while keeping custom, graphql_client
and Cynic request construction and HTTP execution independent. This extends
ADR 47's restriction on shared helpers to permit static retailer configuration.
Each retailer has its own embedded documents and schema. Generated GraphQL
client and Cynic definitions remain isolated by retailer. The API version is
2026-10; Canada uses CA/EN and AU uses AU/EN. Only offer details sends a token.

Keep exactly two calls and two request/response pairs per implementation per
retailer, without retries or automatic pagination. Saved JSON is pretty-printed.
Live tests are opt-in and independently selectable for each retailer. Offline
checks compare each implementation's query, variables, endpoint and headers
against its Bruno collection.

## Consequences

Existing callers must pass `Storefront::AnkerAu` and an output root without a
retailer suffix. Canada callers use `Storefront::AnkerCa`. Existing AU artifact
paths remain unchanged; Canada adds a sibling `ankerca` directory.

Canadian schema, token and fixed product selection require live verification
before successful Canadian artifacts can be claimed. Network failures do not
justify fabricated responses or reduced query selections. This comparison
remains outside the production offer pipeline and does not add French support.
