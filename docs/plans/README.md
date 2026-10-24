# TODO implementation plans

Item numbers follow [TODO.md](../../TODO.md). Status markers mean:
`[ ]` pending, `[.]` planned with implementation pending, and `[x]` completed.
Planning an item does not mark its implementation complete.

## Current round and dependencies

| Item | Work | Status and prerequisites |
| --- | --- | --- |
| 1 | Upgrade sourcing to local Mulac | Completed. [Original plan](01-retailer-sourcing-local-mulac.md). |
| 2 | Customer-facing queues and health | Completed. [Original plan](02-customer-facing-queue-runners.md). |
| 3 | Processing for export | Completed. [Original plan](03-processing-for-export.md). |
| 4 | Record export messages in sourcing outbox | Completed. [Handoff](04-offer-export-outbox-listener.md). |
| 5 | Publish sourcing outbox to offers | Completed. [Original plan](05-retailer-sourcing-http-outbox.md). |
| 6 | Publish offers outbox to customer-facing | Completed. [Handoff](06-offers-http-outbox.md). |
| 7 | Persist customer-facing offers | Planned, ready. Queue infrastructure and upstream event/model contracts exist. [Handoff](07-customer-facing-offer-projection.md). |
| 8 | Paginated REST and GraphQL offers API | Pending. Owned by customer-facing; waits for item 7's model and storage. |
| 9 | Customers homepage with 30 offers | Pending. Waits for item 8's API. |

Items 4, 6, and 7 can be implemented and verified independently. Item 4 needs
no customer-facing work. Item 6 completes at durable inbox acceptance and does
not require item 7. Item 7 can consume directly submitted contract fixtures
without item 6. A combined walkthrough follows once the pieces are available.

The new handoffs specify requirements, current contracts, scope boundaries,
failure expectations, and acceptance evidence. Implementation design and code
organization are left to the implementing tool. Source links provide evidence
of existing behavior, not prescribed edit locations.

Plans 1, 2, 3, and 5 are historical planning documents; some wording predates
their implementation. For implemented behavior, consult current code, service
READMEs, and accepted ADRs rather than treating old provisional statements as
outstanding decisions.

## Agreed cross-service boundaries

Items 6 and 7 contain the same shared message contract. HTTP inbox routing keys
are the exact event types `OfferCreated`, `OfferTitleChanged`, and
`OfferPriceChangedEvent`. Raw event payloads and stable message UUIDs are
preserved. Mulac converts inbox messages to commands; a newly generated command
UUID is distinct from the original message identity.

HTTP replaces offers' active RabbitMQ delivery during this phase. Missing HTTP
configuration leaves publication unavailable; no fallback or dual publication
is requested. [ADR 20](../adr/0020-temporary-http-outbox-delivery.md) covers only
sourcing-to-offers. Item 6 requires a new ADR for the offers-to-customer-facing
change to [ADR 16](../adr/0016-offers-rabbitmq-outbox.md).

Customer-facing must tolerate duplicate replay without regressing already
applied state. Distinct updates apply in processing order; the current events
carry no offer revision. Source-order guarantees and stale-update protection
are outside this round. Updates arriving before creation fail visibly and
follow normal retries rather than creating partial offers.

## Environment and verification

The new plans were grounded at monorepo revision
`5315da8f36e50be3d70d0da7f873f34262992a6c` and sibling Mulac revision
`9474319fff2237d81828a23d97df5d2f97741858`. Local Cargo path dependencies
use the checked-out files, so implementers must record the actual revision
they validate and recheck contracts when either checkout changes.

The services share a Cargo workspace. Coordinate shared dependency and lockfile
changes while keeping each item's acceptance independently verifiable.
Follow applicable `AGENTS.md` instructions, formatting before CI, relevant
isolated PostgreSQL tests, and live service health checks. Audit only when Cargo
dependencies or lockfiles change. Never claim a skipped integration check passed.

Each handoff describes its own completion evidence and operational boundaries.
No background queue workers, downstream runner invocation, historical backfill,
or operational database resets are included in this round.
