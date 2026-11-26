//! Saga orchestration.

mod initial_fill;
mod model;
#[cfg(test)]
mod postgres_tests;
pub mod prelude;
mod record;
mod repository;
mod schema;

pub(crate) trait RecordableSaga {}
