use chrono::DateTime;
use chrono::Utc;

use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

use crate::platform::bus::Command;
use crate::platform::object_storage::OfferStorageMetadata;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessOfferForExport {
    pub acquired_at: DateTime<Utc>,
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<OfferStorageMetadata>,
}

impl Command for ProcessOfferForExport {
    const NAME: &'static str = "ProcessOfferForExport";
}
