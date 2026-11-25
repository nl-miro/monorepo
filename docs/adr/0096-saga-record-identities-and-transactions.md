# 96. Saga record identities and transactions

Status: Accepted

Supersedes ADR 93 for record identity and repository transactions.

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

Listener adoption and saga progress semantics are subsequent delivery items
in the saga implementation plan. This storage change does not enable workers
or claim completion of the saga workflow.
