//! Run the owning retailer's structuralizer against the pinned extracted object.

use crate::platform::bus::{Command, Event, Handle};
use crate::platform::object_storage::{
    EXTRACTED_OFFERS_BUCKET, ObjectStorage, OfferStorageMetadata, STRUCTURALIZED_OFFERS_BUCKET,
};
use crate::steps::prelude::SourceType;
use chrono::{DateTime, Utc};
use retailer_data_extractor::structuralize_v2_for;
use serde::{Deserialize, Serialize};
use shared::prelude::RetailerCode;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StructuralizeOfferDetails {
    pub retailer_code: RetailerCode,
    pub source: SourceType,
    pub storage: OfferStorageMetadata,
    pub fetched_at: DateTime<Utc>,
}

impl Command for StructuralizeOfferDetails {
    const NAME: &'static str = "StructuralizeOfferDetails";
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OfferDetailsStructuralized {
    pub retailer_code: RetailerCode,
    pub source: SourceType,
    pub storage: OfferStorageMetadata,
    pub fetched_at: DateTime<Utc>,
}

impl Event for OfferDetailsStructuralized {
    const NAME: &'static str = "OfferDetailsStructuralized";
}

pub(in crate::steps) struct Handler {
    pub storage: ObjectStorage,
}

impl Handle for Handler {
    type Command = StructuralizeOfferDetails;
    type Event = OfferDetailsStructuralized;
    type Error = String;

    fn handle(&self, cmd: StructuralizeOfferDetails) -> Result<Vec<Self::Event>, String> {
        let input = self
            .storage
            .get_offer(EXTRACTED_OFFERS_BUCKET, &cmd.storage)
            .map_err(|e| e.to_string())?;

        let document = structuralize_v2_for(cmd.retailer_code, &input)?;

        let storage = self
            .storage
            .create_derived_offer(STRUCTURALIZED_OFFERS_BUCKET, &cmd.storage, &document)
            .map_err(|e| e.to_string())?;

        let evt = OfferDetailsStructuralized {
            retailer_code: cmd.retailer_code,
            source: cmd.source,
            fetched_at: cmd.fetched_at,
            storage,
        };

        Ok(vec![evt])
    }
}
