//! Retailer sourcing v2 UI layer: driving adapters (HTTP) that translate
//! requests into application use cases and outcomes back into responses.
//!
//! Only [`prelude`] is public; other contexts reach this layer through it.

mod http;

pub mod prelude;
