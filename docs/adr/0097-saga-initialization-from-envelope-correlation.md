# 97. Saga initialization from envelope correlation

Status: Accepted

The initialization command handler validates storefront support and the context
list, then emits a retailer-only event. It has no repository dependency.

The registered saga listener requires a valid envelope correlation UUID v7.
It creates or recovers a saga under the type/correlation uniqueness constraint
and returns commands only after the transaction commits. Its independent record
ID supplies the namespace for stable token and context-specific catalog IDs.

The saga payload contains the requested contexts, token policy, and storefront
URL captured at initialization. It contains no duplicated record ID or retailer.
Replay first loads that saved snapshot and never rereads live inventory when a
record already exists. Concurrent first events converge on one persisted
snapshot. A conflicting retailer or malformed correlation fails explicitly.

This implements the initialization boundary. Subsequent step listeners and
removal of step payload run IDs remain tracked in the implementation plan.
