# testimony

Event sourcing abstractions: aggregates, event envelopes, event stores,
publishers, and repositories that load and save aggregate histories. Also
provides event metadata and query dispatch through query handlers and a query
bus.

Import common types from `testimony::prelude`. Concrete event stores live in [`testimony_infra`](../testimony_infra/README.md).

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

## Fallible history loading

`Aggregate` implementors must define `HistoryError: Error + Send + Sync + 'static`
and return `Result<Self, Self::HistoryError>` from `from_history`. This is a
breaking Rust trait change; callers must handle replay failures. Successful
replay restores state and sequence without recording uncommitted events.

`EventSourcedRepository::load` still sorts envelopes by sequence and returns
`Ok(None)` for an absent stream. Replay failures return `RepositoryError::History`
with an `Arc` preserving clone support and the concrete error through `source()`.
History errors are propagated directly, without catching panics. Event formats
and repository save behavior are unchanged.
