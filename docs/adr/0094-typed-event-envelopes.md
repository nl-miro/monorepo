# 94. Typed event envelopes in retailer sourcing

Status: Accepted

Saga listeners need both the decoded event and its execution metadata.
The sourcing bus exposes `EventEnvelope<E>` with an `event` field and the
envelope's optional metadata, including sequence, event ID, correlation ID,
causation ID and source.

`Bus::on_commands_envelope` registers callbacks that receive this wrapper.
The adapter validates the event type and decodes its payload before calling
the callback. It accepts closures so registrations can capture listener
instances and their repositories.

`Bus::on_commands` delegates through the same adapter and supplies the decoded
event to its existing callbacks. Command dispatch continues to use each
returned command's correlation ID and the incoming event ID as causation.

The Shopify GraphQL listener methods receive typed envelopes. Establishing
a stable parent-saga correlation for catalog events is a separate decision.
