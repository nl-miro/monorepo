# Export conversion fixtures

Small synthetic examples preserve the structuralized document shapes emitted by
Anker AU, Ugreen CA and Minisforum US. They cover variant URLs, unrelated fields,
optional prices and unavailable stock without relying on downloaded retailer data.

`import_payload.json` is the current export payload for the first Anker
variation, including its `acquired_at`. Sourcing checks its serialized result
against this fixture, offers deserializes it through its actual
`ImportOfferDetailsCommand`, and Customer Facing projects it. Run all three
services' `make ci` checks when changing this contract.
