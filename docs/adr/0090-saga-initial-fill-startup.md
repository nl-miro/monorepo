# 90. Start initial fills through the saga service

Status: Accepted

Supersedes ADR 79 for initial-fill startup and its HTTP contract.

## Decision

`POST /api/initial-fill` accepts a JSON array of retailer codes. The saga
service dispatches one `RequestInitialFill` per entry. Saga listeners select
the next commands from the resulting events. The HTTP adapter returns an empty
`202` when the service returns, `422` for a JSON value of the wrong shape,
and a JSON error with `503` when the command bus is unavailable.

## Consequences

Callers receive no per-retailer run IDs or partial-success report. The service
discards dispatch errors, so acceptance does not confirm durable queueing.
The registered handlers continue to read the existing command and event formats.
