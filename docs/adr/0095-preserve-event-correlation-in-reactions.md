# 95. Preserve event correlation in command-producing reactions

Status: Accepted

This supersedes ADR 94's command-correlation behavior. Its typed event-envelope
API remains unchanged.

All retailer-sourcing event reactions copy the incoming event's correlation ID
into the commands they produce. `on_many_keyed`, `on_commands`, and
`on_commands_envelope` no longer replace it with a command's run ID. This keeps
one correlation across catalog contexts and subsequent processing steps.

Command IDs retain their existing stable derivation for deduplication. Run IDs
retain their application meaning. Causation remains the incoming event ID.
An absent correlation stays absent at this boundary, matching ordinary reactions.
Mulac's existing command-to-event fallback to command ID is unchanged.

This applies to newly dispatched commands. Existing queued commands whose
correlation was replaced are not rewritten by this change. Saga lookup and
correlation uniqueness are separate work.
