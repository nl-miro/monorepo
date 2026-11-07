//! Records one outbox message per completed export variation for delivery
//! to other services on request. Other contexts access this module via [`prelude`].

pub mod prelude;

mod record_export_messages;
