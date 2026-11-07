# Export conversion fixtures

Small synthetic structuralized documents for Anker AU, Ugreen CA and Minisforum
US cover variant URLs, optional prices and unavailable stock.

All import fixtures use the current unversioned export format:

- `import_payload.json`: the exact first Anker export, with images and optional facts.
- `import_with_identifiers.json`: all five identifiers, with exact source values.
- `import_with_facts.json`: comparison price, availability, item condition and description.

Tests compare serialized exports directly with these fixtures. Run
`make -C services/retailer_sourcing ci` when changing this contract.
