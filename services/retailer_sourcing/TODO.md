## Open Questions

- [ ] Does `Runner::start_fills` in the Shopify GraphQL initial fill saga need to be async?
  Could the runner dispatch all commands for a request in one atomic bulk insert so
  the database persists either every command or none of them?
- [ ] For each saga and step, have we documented every way it can fail?
- [ ] Should we support inserting commands in bulk?

## Planned work

- [>] [Define correlation-key uniqueness and migration policy.](../../docs/plans/saga-implementation.md#1-uniqueness-and-migration)
- [>] [Add correlation storage and repository lookup.](../../docs/plans/saga-implementation.md#2-repository-and-record)
- [>] [Expose incoming command context to initialization.](../../docs/plans/saga-implementation.md#3-initialization-receives-command-metadata)
- [x] [Preserve correlation through saga dispatch.](../../docs/plans/saga-implementation.md#4-preserve-correlation)
- [>] [Register saga listeners using correlation lookup.](../../docs/plans/saga-implementation.md#5-listener-lookup-and-registration)
- [>] [Verify and document saga correlation lookup.](../../docs/plans/saga-implementation.md#6-verification-documentation-and-release)
