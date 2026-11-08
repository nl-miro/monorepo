# ADR 47: Fixed GraphQL operations

Status: Accepted

Supersedes [ADR 46](0046-independent-graphql-clients.md).

## Context

The comparison now requires two fixed queries per implementation: catalog
offers and offer details. The former schema-driven catalogue downloader's
`Field` and `Dump` structures, discovery, retries and pagination are unnecessary.

## Decision

Keep the custom, graphql_client and Cynic clients, each building and sending
the two operations independently. Each call sends one HTTP request and saves
one request/response pair under a stable implementation/operation directory.
The clients share only the error alias and JSON file-writing helper.

Remove the old downloader, its public download functions, CLI, catalogue
comparison tests and schema-discovery fixtures. Retain the checked-in schema
used by graphql_client and Cynic to compile their fixed operations, the Bruno
query parity test, and the six-request live comparison.

## Consequences

The clients no longer discover fields, follow connections, retry requests or
assemble catalogue exports. Responses preserve the server's pagination metadata
and contain only the requested page. Running the live comparison replaces the
six saved request/response pairs. This remains outside the production offer
pipeline.
