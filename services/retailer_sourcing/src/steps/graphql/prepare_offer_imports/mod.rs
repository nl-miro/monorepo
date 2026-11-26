//! Record the structuralized artifact's import messages for the offers inbox.

use super::structuralize_offer_details::OfferDetailsStructuralized;
use crate::platform::bus::{Event, EventContext, Listen};
use crate::platform::db::DbPool;
use crate::platform::object_storage::{ObjectStorage, STRUCTURALIZED_OFFERS_BUCKET};
use crate::platform::outbox::{OutgoingMessage, record};
use crate::steps::prelude::shopify_imports;
use serde_json::to_string;
use uuid::Uuid;

pub(in crate::steps) struct Listener {
    pub(in crate::steps) pool: DbPool,
    pub(in crate::steps) storage: ObjectStorage,
}

impl Listen for Listener {
    type Event = OfferDetailsStructuralized;
    type Error = String;
    const NAME: &'static str = "PrepareOfferDetailsImports";

    fn listen(&self, event: Self::Event, context: &EventContext) -> Result<(), String> {
        let source = &event.storage;
        let body = self
            .storage
            .get_offer(STRUCTURALIZED_OFFERS_BUCKET, source)
            .map_err(|e| e.to_string())?;
        let imports = shopify_imports(&body, event.retailer_code, source.acquired_at)
            .map_err(|e| e.to_string())?;
        let mut messages = Vec::with_capacity(imports.len());
        for (variant_id, import) in imports {
            messages.push(OutgoingMessage {
                id: Uuid::new_v5(&source.object_storage_id, &variant_id.to_be_bytes()),
                payload: to_string(&import).map_err(|e| e.to_string())?,
                routing_key: "offers",
                event_type: OfferDetailsStructuralized::NAME,
            });
        }
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        record(&mut conn, &messages, context).map_err(|e| e.to_string())
    }
}
