# Saga implementation — retailer initial fill

The `retailer_initial_fill` business process collects a retailer's initial set
of offers during onboarding. The Shopify GraphQL saga owns the sequence of
steps for that collection method. Step handlers do the work and emit outcome
events. Other sagas can reuse those handlers. [ADR 92](../adr/0092-business-processes-sagas-and-steps.md)

## Starting a Shopify GraphQL fill today

`POST /api/initial-offer-fills` accepts a body such as
`{"retailers":["ankerau","ankerca"]}`. The route starts the GraphQL saga.
A storefront's `preferred` strategy can be `ShopifyGraphQl`,
`ShopifySitemapPlusHtml`, `Cafe24`, `HtmlMenuCatalogDetails`, or
`SitemapThenHtmlDetails`. Only `ShopifyGraphQl` makes
`config.access.supports_graphql()` true. The route rejects other preferences
and missing configurations. [Retailer integration guide](../retailer-integration.md#graphql-initial-fills)

1. The Axum handler `start_initial_fills` extracts the JSON request, converts exact retailer slugs to typed codes, and validates the batch. Unknown slugs are reported in request order, including repeated occurrences. Once all slugs are known, validation checks for an empty list, then duplicates, then GraphQL eligibility. Duplicate retailers appear once at their first position. A failing stage returns immediately.
2. For a valid batch, the handler calls `build_controller(bus)`, then `controller.start(retailers)`. The builder requires a bus, constructs `Service` with its command gateway, and returns `InitialFillController::new(service)`. A missing bus causes a panic before dispatch.
3. The controller passes the validated codes to `Service::run` in request order. The service creates and dispatches one `InitializeRetailerFill` command at a time. Each command gets its own UUID v7, which also serves as its flow correlation. The saga record receives a separate ID later.

| HTTP result | What the caller can conclude |
| --- | --- |
| `204 No Content` | All initialization commands were queued. The response has no body. |
| `400 Bad Request` for retailer validation | Nothing was queued. An empty array returns `{"error":"retailers must not be empty","failures":[]}`. Other validation failures return `invalid retailers` with a reason of `unknown`, `duplicate`, or `not_graphql` for each reported entry. |
| `500 Internal Server Error` during dispatch | The service stopped at the first error. Earlier commands stay queued. Retrying the entire POST starts fresh correlations and can repeat those fills. |

Axum handles JSON extraction errors before the controller runs. [Axum handler and validation](../../services/retailer_sourcing/src/saga/shopify_graphql_initial_fill/ui/http/axum.rs) · [Initialization service](../../services/retailer_sourcing/src/saga/shopify_graphql_initial_fill/service.rs)

## Current flow from queued command to completed saga

1. **Initialization.** The command handler loads the storefront configuration, validates its contexts, and emits `RetailerFillInitialized` with the retailer code. The event listener creates or recovers the saga using its type and the envelope correlation. On first creation it saves the requested contexts, token decision, and storefront URL. An initialization command can therefore fail before a saga row exists. [Handler](../../services/retailer_sourcing/src/steps/graphql/initialize_retailer_fill.rs) · [ADR 97](../adr/0097-saga-initialization-from-envelope-correlation.md)
2. **Token and contexts.** The saga queues `AcquireStorefrontToken` when the saved configuration calls for acquisition. Otherwise it immediately queues one `FetchProductCatalogIdentifiers` per distinct requested context. A context has a language and an optional country. Anker Canada uses CA/EN and CA/FR. Minisforum EU uses EN with country omitted. An empty locale list or a region in the country field fails context validation. [Context derivation and examples](../../services/retailer_sourcing/src/v2/incoming.rs) · [Saga model](../../services/retailer_sourcing/src/saga/shopify_graphql_initial_fill/model.rs)
3. **Catalogs and details.** Each catalog handler follows pagination to completion and emits one `ProductCatalogIdsRetrieved` with deduplicated product IDs. That outcome can immediately start one `FetchOfferDetails` per product for its context while other catalogs are still running. Storing the fetched document leads to extraction and then structuralization. [Catalog handler](../../services/retailer_sourcing/src/steps/graphql/product_catalog_ids/handler.rs) · [Saga listener](../../services/retailer_sourcing/src/saga/shopify_graphql_initial_fill/listener.rs)
4. **Progress and completion.** Each transition locks the saga row and commits progress before returning its next commands for dispatch. Replay uses saved results and stable command IDs to recover interrupted dispatch. A fill succeeds when every catalog succeeds and every discovered product/context reaches structuralization. Empty catalogs can succeed. A failure or explicit skip makes the terminal fill fail. Import preparation runs independently. [ADR 98](../adr/0098-saga-progress-replay-and-failure-observation.md)
5. **Driving the queues.** The executable starts zero background command workers. Run the command and event batches to advance the saga. Startup and completed manual batches reconcile exhausted work. The saga row's status and payload show progress once initialization has created it. [Service startup](../../services/retailer_sourcing/src/main.rs) · [Saga walkthrough](../../services/retailer_sourcing/src/saga/shopify_graphql_initial_fill/shopify_graphql_initial_fill.md)

## Planned changes to the start contract

This version uses Mulac Commanding’s existing single-command dispatch. If the second retailer’s command fails to queue, the first stays queued and dispatch stops.

- **Saga creation in the command handler.** When a worker runs `InitializeRetailerFill`, its handler uses the command’s envelope correlation to create the saga and save its initial state through `SagaRepository`. Only after the save commits does the handler return one `RetailerFillInitialized` event. The listener uses the event’s correlation to load the saved saga and asks it for the first token or catalog commands. [ADR 99](../adr/0099-command-handler-creates-initial-fill-saga.md)
- **Ordered receipts.** Once every dispatch succeeds, have the service return one retailer/correlation pair per command in request order. The planned HTTP response is `202 Accepted`, for example `{"accepted":[{"retailer":"ankerau","correlation":"019b1234-5678-7000-8000-000000000001"}]}`. The correlation must match the stored command metadata. Update the route schema, client examples, and HTTP and PostgreSQL tests together.
- **Validation across categories.** The planned response reports one entry per retailer and reason in first-seen order. Cover a batch containing an unknown slug, a known duplicate, and a GraphQL-ineligible storefront. Today's staged checks expose only the first failing category.
- **Shared eligibility.** Apply the GraphQL preference rule to initialization commands submitted directly to the queue. The command handler currently checks configuration and contexts, while the HTTP handler checks configuration and preference. The plan also calls for detecting invalid contexts at HTTP admission. Define the failure reason for that case.

## Decisions still open

- The earlier proposal required a country for every locale. Enforcing it would exclude the existing Minisforum EU fill. Decide whether to change the accepted optional-country policy before introducing HTTP context validation. [ADR 74](../adr/0074-drop-market-discovery-stage.md)
- Decide how to record and verify each storefront's preference in the inventory, including the role of `graphql_api_status`. Selection and fallback across different saga implementations remain open.
- Decide whether repeated submissions for one retailer should reuse a run, whether overlapping runs are acceptable, and how callers should retrieve progress from a receipt.
- Define any common API contract for malformed JSON and any further policy for timeouts, resuming a failed run, or refreshing an expired token. The current replay policy retains the original results and token. [ADR 98](../adr/0098-saga-progress-replay-and-failure-observation.md)

The [original walkthrough](saga-chat-transcript.md) records the onboarding example that led to this plan.
