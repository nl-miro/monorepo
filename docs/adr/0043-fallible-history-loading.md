# 43. Fallible aggregate history loading

## Status

Accepted

## Date

2026-11-01

## Context

Offers history replay used assertions and panics for malformed streams. The
Testimony aggregate contract returned `Self`, preventing repositories from
reporting replay failures normally. Next-generation replay was unimplemented.

## Decision

Require an associated `HistoryError: Error + Send + Sync + 'static` and return
`Result<Self, Self::HistoryError>` from `Aggregate::from_history`. Repositories
wrap failures in `RepositoryError::History(Arc<dyn Error + Send + Sync>)` and
expose the original error through `source()`, preserving clone support.
Repositories retain sorting and absent-stream behavior.

Both Offers aggregates use a shared typed history error. Replay validates stream
structure and identity, rejects unsupported events, and produces no uncommitted
events. Legacy replay retains its existing payload validation. Next-generation
replay initializes once through `when_created` and checks each event with a
read-only guard before calling `when`. The guard rejects legacy events and unsafe
image insertion positions. It adds no business validation, normalization,
previous-value comparisons, or timestamp ordering requirements. Sequence
increments use checked arithmetic; errors are not translated from panics.

## Consequences

This breaks the Rust trait API: implementors must supply an error type and direct
callers must handle `Result`. Stored event formats remain unchanged. Production
continues to use the legacy aggregate. Next-generation command methods and the
two remaining trait stubs are outside this decision's implementation scope.
