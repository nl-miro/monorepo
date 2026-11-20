# AGENTS.md

Read [README.md](README.md) for the service layout and requirements.

## Architecture

- Application code implements use cases by using domain code and infrastructure
  implementations.
- Domain code owns business concepts and rules.
- Infrastructure code implements access to databases, external services, storage,
  and other technical capabilities used by the application.
- UI is an entry point into the application: HTTP, CLI, or another interface.
  It translates input into application calls and maps their results into output.
  Keep use-case orchestration in application code and business rules in domain
  code.

## Business processes, sagas and steps

- In `src/business_process/`, name each process for the outcome the business
  wants. For example, `retailer_initial_fill` collects a new retailer's initial
  set of offers when the retailer comes on board.
- In `src/saga/`, define how to achieve that outcome. The Shopify GraphQL saga
  and the sitemap-and-HTML saga are two implementations of the same retailer
  initial fill. Each saga calls the steps its approach requires, in order.
- In `src/steps/`, define each step as one small action, such as retrieving a
  sitemap or downloading an offer page. Reuse a step across sagas and business
  processes whenever they need the same action.

See [ADR 92](../../docs/adr/0092-business-processes-sagas-and-steps.md) for the
retailer onboarding example.

## v2 code style

- Place submodule declarations (`mod xyz;`) at the top of the file, after
  module documentation and before production imports.
- In `src/v2`, import functions, types and macros with `use` instead of using
  module-qualified paths inline. Use descriptive import aliases when names
  collide. Keep associated calls and enum variants qualified by their type
  (for example, `Uuid::now_v7()` and `SourceType::GraphQl`).
- Keep test-only imports inside the test module. For tests in the same module
  as production code, place their imports just above the first test.
- In `src/v2`, when a returned constructed value takes more than 4 lines, define
  it with `let` before returning it. Shorter ones may be returned directly.
  A value that is itself the returned expression is exempt, because clippy's
  `let_and_return` flags binding it. Don't add `#[allow(clippy::let_and_return)]`.
- This doesn't apply when returning a property or variable straight out, since
  nothing is constructed. It also doesn't apply to a constructor that only
  assigns its own parameters to fields, like `Self { run_id, url }`.
