# Saga review fixes

- [x] Subscriber order and import failure isolation.
- [x] Start validation, correlation receipts and blocking dispatch.
- [x] Coordinated stale-reservation recovery.
- [x] Per-saga reconciliation failure isolation.
- [x] Queue correlation indexes and unchanged-write avoidance.
- [x] Expected-work matching before retailer validation.
- [x] Consolidated decisions and consistent documentation.

## Open questions

- [ ] Should multi-retailer initialization use atomic bulk insertion?
