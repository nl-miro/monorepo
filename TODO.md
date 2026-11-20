# TODO

Status follows the [TODO workflow](AGENTS.md#todo-workflow):

| Marker | Phase                      |
| ------ | -------------------------- |
| `[ ]`  | Pending                    |
| `[.]`  | Planning in progress       |
| `[>]`  | Planning complete          |
| `[~]`  | Implementation in progress |
| `[x]`  | Implemented                |

Flow: `[ ]` → `[.]` → `[>]` → `[~]` → `[x]`.

- [x] Move the three Shopify GraphQL initial-fill steps into the root `steps/graphql` module.

- [x] Move sitemap retrieval, processing and consolidation into `steps/sitemap` and retire the HTTP retrieval endpoint.

Shopify GraphQL initial-fill saga through structuralized offer details, in the
[implementation plan](docs/plans/saga-implementation.md)'s delivery order:

- [x] Saga storage: consolidate the unshipped migrations into one, and add the saga record with its payload state, uniqueness, PostgreSQL 18 UUID checks, and locked transaction-scoped transitions. Move CI to PostgreSQL 18.

- [x] Initialization: move saga creation into the `RetailerFillInitialized` listener, simplify the handler and event, and remove duplicated identity from the saga payload.

- [x] Step contracts and command identity: expose reusable step contracts and context-keyed builders that use the persisted context snapshot and the internal saga ID namespace. Remove payload and `KeyedCommand` `run_id`, and move metadata consumers to envelope correlation.

- [x] Transitions and progress: implement transitions, canonical results, payload progress state, and replayed dispatch. Register the saga listeners and remove duplicate dispatch owners.

- [x] Failure reconciliation: reconcile exhausted commands and saga event deliveries into terminal outcomes, and evaluate the fill against the Success Criteria.

- [x] End-to-end verification: complete queue and migration verification, then run a token and a tokenless retailer through the API to their final outcomes and inspect the structured artifacts.
