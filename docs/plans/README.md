# TODO implementation plans

These plans cover the unchecked items in [TODO.md](../../TODO.md). They describe
implementation work; no checklist item has been implemented or marked complete.
Item numbers below refer to the current order of that checklist.

The final selection awaits clarification of “not dependent”: an implementation
can sometimes be built and tested against an existing contract before the whole
pipeline can run. The four drafts linked below cover work with no prerequisite
code changes from another TODO item. Product choices inside a draft remain
explicitly pending where indicated.

## Dependency assessment

| Item | Work | Prerequisites and current plan |
| --- | --- | --- |
| 1 | Upgrade retailer sourcing to local Mulac | No TODO prerequisite. [Plan](01-retailer-sourcing-local-mulac.md). |
| 2 | Set up customer-facing queues and health | No TODO prerequisite. Health already exists; database startup and runner scope await interpretation. [Plan](02-customer-facing-queue-runners.md). |
| 3 | Processing for export | Existing structuralization events and bus interfaces suffice; item 1 is coordination, not a prerequisite. Export shape and legacy handling await interpretation. [Draft plan](03-processing-for-export.md). |
| 4 | Record export events in the sourcing outbox | Needs item 3's export event and payload contract. |
| 5 | Publish sourcing outbox messages to offers | Publisher implementation can use seeded rows and the existing offers inbox. Item 4 supplies real pipeline messages. Local Mulac dependency edits overlap item 1. [Provisional plan](05-retailer-sourcing-http-outbox.md); final inclusion awaits the dependency interpretation. |
| 6 | Publish offers outbox messages to customer facing | Existing publisher interface and outbox support implementation against an HTTP test receiver. Live delivery needs item 2; applying events needs item 7. The inbox event mapping must be agreed. Inclusion awaits the dependency interpretation. |
| 7 | Customer-facing offer entity and record | Entity, record, migration and database mappings can be implemented independently. Receiving and persisting events, as required by the TODO note, additionally needs item 2 and a matching inbox event contract. Inclusion awaits the intended completion boundary. |
| 8 | Paginated REST and GraphQL offers API | If owned by customer facing, needs item 7's model and storage. If owned by offers, its read model already exists. Service ownership awaits interpretation. |
| 9 | Customers homepage showing 30 offers | Needs item 8's retrieval contract and service; a fixture-only design would not complete the checklist item. |

The source for this assessment is the current implementation, including
[sourcing's bus](../../services/retailer_sourcing/src/platform/bus.rs),
[its structuralization stage](../../services/retailer_sourcing/src/offer_discovery/structuralize_offer/handler.rs),
[offers' inbox routes](../../services/offers/src/orchestration/inbox_api.rs),
[its outbox subscriber](../../services/offers/src/subscribers/customer_facing.rs),
and the [customer-facing server](../../services/customer_facing/src/main.rs).
Older offers documentation also describes planned behavior that is not active.

## Coordination across plans

The services share one Cargo workspace. Centralize reused local Mulac
dependencies and coordinate manifest/lockfile changes so items 1 and 2 remain
independently implementable. Neither plan should assume the other has landed.
Record the sibling Mulac revision used for implementation and validation.

Both temporary HTTP publishers depart from
[ADR 16](../adr/0016-offers-rabbitmq-outbox.md), as the TODO explicitly allows.
Their implementation should add a new ADR describing the temporary transport,
durable acceptance, retries, duplicate handling and eventual broker migration.
Do not edit the accepted ADR to erase the earlier decision.

Offers currently places event type in outbox metadata and uses
`customer_facing` as its routing key. Mulac's inbox turns routing keys into
command types and does not automatically preserve event type. Plans for items
6 and 7 must specify the same event wrapper or routing translation; accepting an
HTTP inbox message alone does not persist a customer-facing offer.

Each completed implementation must follow the repository checks: formatting
before CI, dependency audits when Cargo dependencies or lockfiles change,
relevant PostgreSQL tests, and live health/page checks for changed services or
apps. The plan documents distinguish those future checks from the work of
writing the plans.
