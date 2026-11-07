# ADR 45: Local Mulac path dependencies

Status: Accepted; supersedes ADR 42's published-Mulac dependency decision.

Date: 2026-11-03

## Context

Development now needs changes in the sibling Mulac checkout to be consumed
without publishing new packages. All direct consumers are in the services
workspace; the libraries and apps do not directly depend on Mulac.

## Decision

Use Cargo path dependencies pointing to `../mulac` from the monorepo root.
Keep shared dependencies in `services/Cargo.toml` and retain the existing
feature sets and Rust dependency names. The local API packages are named
`commanding_api`, `eventing_api`, `inbox_api` and `outbox_api`, without the
published `mulac_` prefix.

The baseline checkout is `nl-miro/mulac` at
`fd29f72a5b73ff5549440e547f0a0a50a705a504`, whose manifests use version 0.3.0.
CI checks out that revision beside the monorepo in each job. Commit the
services lockfile with local package resolution. Unrelated registry
dependencies retain their existing sources.

## Consequences

Builds need the sibling checkout, including its `kernel` and `libs` folders.
Local Mulac edits take effect immediately. Cargo.lock does not pin the contents
of path dependencies; CI's checkout revision must be updated intentionally
when the required baseline changes. This change does not publish Mulac or
modify the sibling repository.

## Verification

Use `cargo metadata --offline --format-version 1` in `services` to verify that
all Mulac packages resolve to the sibling checkout with no registry source.
Run formatting, CI and dependency audit for the services workspace, and service
startup checks. Database tests require a disposable PostgreSQL database.
