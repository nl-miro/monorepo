# 93. Saga storage and identity

Status: Accepted

Consolidates the branch-only saga storage decisions.

Saga records have an independent UUID v7 primary key and an original flow
correlation UUID v7, unique with the saga type. Different types may share a
correlation, but cannot share a primary key. The retailer is validated after
lookup so a conflicting owner cannot be mistaken for a missing record.

The record stores `in_progress`, `succeeded`, or `failed` and a JSON object
payload. PostgreSQL 18 checks UUID version and variant with
`COALESCE(uuid_extract_version(value), 0) = 7`. Rust boundaries validate both
identities and status too.

The repository provides a create-or-load transition that uses the unique
correlation constraint and `SELECT ... FOR UPDATE` on one connection. Creation,
callback changes, and saving commit together. Callback errors roll back the
transaction. Identity changes and reopening a terminal status are rejected.

The three unshipped saga migrations are consolidated into one create/drop
migration. This requires a fresh database; existing development databases
must be explicitly recreated before using this migration history.

Step events match expected work before validating the incoming retailer. Initialization always validates ownership. Unchanged transitions retain the row without rewriting its payload; replay still returns the saved commands.
