//! Parses a retrieval's stored sitemap files into one sitemap tree and stores
//! it.

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
