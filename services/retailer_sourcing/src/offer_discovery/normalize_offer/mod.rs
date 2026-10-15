//! Normalizes an offer's extracted JSON with the retailer's normalizer, and
//! stores it.

pub mod prelude;

mod command;
mod entity;
mod event;
mod handler;
#[cfg(test)]
mod postgres_tests;
mod reacts_to;
mod record;
mod store;
