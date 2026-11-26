//! Domain input and output for this feature slice.
use chrono::DateTime;
use chrono::Utc;

use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ExtractedOffer {
    pub acquired_at: DateTime<Utc>,
    pub object_storage_id: Uuid,
    pub offer_id: Uuid,
    pub document: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct NormalizedOffer {
    pub acquired_at: DateTime<Utc>,
    pub offer_id: Uuid,
    pub document: Value,
}
