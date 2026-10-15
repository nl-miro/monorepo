//! Structuralizes an offer's normalized document with the retailer's
//! structuralizer, and stores it.

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
