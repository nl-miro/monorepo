//! Extract the retailer-owned model from a stored initial-fill artifact.

use crate::platform::bus::{Command, Event, Handle};
use crate::platform::object_storage::{
    EXTRACTED_OFFERS_BUCKET, ObjectStorage, OfferStorageMetadata, RAW_OFFERS_BUCKET,
};
use crate::steps::prelude::SourceType;
use chrono::{DateTime, Utc};
use retailer_data_extractor::extract_v2_for;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, to_value};
use shared::prelude::RetailerCode;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ExtractOfferDetails {
    pub retailer_code: RetailerCode,
    pub source: SourceType,
    pub storage: OfferStorageMetadata,
    pub fetched_at: DateTime<Utc>,
}

impl Command for ExtractOfferDetails {
    const NAME: &'static str = "ExtractOfferDetails";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OfferDetailsExtracted {
    pub retailer_code: RetailerCode,
    pub source: SourceType,
    pub storage: OfferStorageMetadata,
    pub fetched_at: DateTime<Utc>,
}

impl Event for OfferDetailsExtracted {
    const NAME: &'static str = "OfferDetailsExtracted";
}

pub(in crate::steps) struct Handler {
    pub(in crate::steps) storage: ObjectStorage,
}

impl Handle for Handler {
    type Command = ExtractOfferDetails;
    type Event = OfferDetailsExtracted;
    type Error = String;

    fn handle(&self, cmd: ExtractOfferDetails) -> Result<Vec<Self::Event>, String> {
        let raw = self
            .storage
            .get_offer(RAW_OFFERS_BUCKET, &cmd.storage)
            .map_err(|e| e.to_string())?;
        let mut artifact: Value = from_str(&raw).map_err(|e| e.to_string())?;
        artifact
            .as_object_mut()
            .ok_or("expected a raw offer object")?
            .entry("acquired_at")
            .or_insert(to_value(cmd.fetched_at).map_err(|e| e.to_string())?);
        let extracted = extract_v2_for(cmd.retailer_code, &artifact.to_string())?;
        let storage = self
            .storage
            .create_derived_offer(EXTRACTED_OFFERS_BUCKET, &cmd.storage, &extracted)
            .map_err(|e| e.to_string())?;
        Ok(vec![OfferDetailsExtracted {
            retailer_code: cmd.retailer_code,
            source: cmd.source,
            fetched_at: cmd.fetched_at,
            storage,
        }])
    }
}
