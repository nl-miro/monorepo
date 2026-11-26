#![allow(dead_code)]

//! Extract the retailer-owned model from a stored initial-fill artifact.

use crate::platform::bus::{Command, Event, Handle};
use crate::platform::object_storage::{
    EXTRACTED_OFFERS_BUCKET, ObjectStorage, OfferStorageMetadata, RAW_OFFERS_BUCKET,
};
use crate::steps::prelude::SourceType;
use chrono::{DateTime, Utc};
use retailer_data_extractor::extract_v2_for;
use serde::{Deserialize, Serialize};
use shared::prelude::RetailerCode;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ExtractOfferDetails {
    pub retailer_code: RetailerCode,
    pub source: SourceType,
    pub storage: OfferStorageMetadata,
    pub acquired_at: DateTime<Utc>,
}

impl Command for ExtractOfferDetails {
    const NAME: &'static str = "ExtractOfferDetails";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OfferDetailsExtracted {
    pub retailer_code: RetailerCode,
    pub source: SourceType,
    pub storage: OfferStorageMetadata,
    pub acquired_at: DateTime<Utc>,
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
        let raw = self.get_raw_offer(&cmd.storage)?;
        let extracted = extract_v2_for(cmd.retailer_code, &raw)?;
        let storage = self
            .storage
            .create_derived_offer(EXTRACTED_OFFERS_BUCKET, &cmd.storage, &extracted)
            .map_err(|e| e.to_string())?;
        Ok(vec![OfferDetailsExtracted {
            retailer_code: cmd.retailer_code,
            source: cmd.source,
            acquired_at: cmd.acquired_at,
            storage,
        }])
    }
}
impl Handler {
    // Added get_raw_offer method
    fn get_raw_offer(&self, storage: &OfferStorageMetadata) -> Result<String, String> {
        self.storage
            .get_offer(RAW_OFFERS_BUCKET, storage)
            .map_err(|e| e.to_string())
    }
}

#[derive(Deserialize)]
struct RawAcquisition {
    acquired_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::test_objects::Server;
    use crate::steps::graphql::offer_details::stamp_acquired_at;
    use serde_json::from_str;
    use serde_json::{Value, json};

    fn acquired_at() -> DateTime<Utc> {
        "2026-11-12T09:10:11.123456Z".parse().unwrap()
    }

    fn artifact() -> String {
        let product: Value = from_str(include_str!(
            "../../../../tests/fixtures/anker/ankerau_product.json"
        ))
        .unwrap();
        let context = json!({"country":"AU","language":"EN","currency":"AUD"});
        json!({"schema_version":2, "context":context, "product":product, "pages":[]}).to_string()
    }

    fn extract(raw: &str, expected: DateTime<Utc>) -> Result<Vec<OfferDetailsExtracted>, String> {
        let server = Server::new();
        let storage = server.client();
        let stored = storage
            .create_offer(RAW_OFFERS_BUCKET, raw, expected)
            .unwrap();
        Handler { storage }.handle(ExtractOfferDetails {
            retailer_code: RetailerCode::AnkerAu,
            source: SourceType::GraphQl {
                product_id: "gid://shopify/Product/1".into(),
            },
            storage: stored,
            acquired_at: expected,
        })
    }

    #[test]
    fn extracts_the_stamped_raw_body_unchanged() {
        let raw = stamp_acquired_at(&artifact(), acquired_at()).unwrap();
        let event = extract(&raw, acquired_at()).unwrap().remove(0);
        assert_eq!(event.acquired_at, acquired_at());
        assert_eq!(event.storage.acquired_at, acquired_at());
    }
}
