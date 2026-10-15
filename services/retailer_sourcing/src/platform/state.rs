//! What the HTTP routes share.

use axum::extract::FromRef;

use super::bus::BusHandle;
use super::db::DbPool;

/// The routes' shared state. Both are `None` without a database.
#[derive(Clone, Default)]
pub struct AppState {
    /// Queues and runs commands and events.
    pub bus: Option<BusHandle>,
    /// Reads what the pipeline stored.
    pub db: Option<DbPool>,
}

impl FromRef<AppState> for Option<BusHandle> {
    fn from_ref(state: &AppState) -> Self {
        state.bus.clone()
    }
}

impl FromRef<AppState> for Option<DbPool> {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
