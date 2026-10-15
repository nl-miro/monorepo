//! Sorts every URL of a processed sitemap by kind of page (product, catalog,
//! content and so on) using the retailer's link rules, and stores the groups.

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
