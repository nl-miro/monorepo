# ADR 44: Unversioned sourcing exports

## Status
Accepted. Supersedes the export-version requirements of ADRs 19, 24, 26, 28,
29, 34, 35, 36 and 40. Their other decisions remain unchanged.

## Date
2026-11-01

## Context
Sourcing supports one current export format. A version field no longer selects
behavior and unnecessarily participates in preparation identity.

## Decision
Remove schema versions from export commands, snapshots, completion events and
database records. Identify preparations by offer ID and source object ID.
Keep payload validation, source matching, stable message identities and exact
prepared bytes across retries. Preserve existing unknown-field policies.

Update the original export-table migration for fresh databases. Do not introduce
historical readers, convert existing artifacts or reset databases automatically.

## Consequences
A source has one export preparation. Payloads carry no version discriminator.
Existing databases and queued versioned artifacts are outside this fresh-database
contract. PostgreSQL JSONB encoding and migration identifiers are unaffected.
