use serde_json::{Error as JsonError, to_string};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

use super::outbox_repository::{OutboxRecordingError, OutboxRepository};
use crate::offer_discovery::prelude::{
    OfferExportRepository, OfferExportRepositoryError, OfferExportWasPrepared,
};
use crate::platform::bus::{Event, EventContext, Listen};
use crate::platform::outbox::OutgoingMessage;

/// The offers service's inbox destination.
const ROUTING_KEY: &str = "offers";

#[derive(Debug, Error)]
pub(super) enum ListenerError {
    #[error("offer export {0} is not completed")]
    NotCompleted(Uuid),
    #[error("offer export {0} does not match the event's source or payload")]
    Mismatch(Uuid),
    #[error("offer export {0} payload serialization failed: {1}")]
    Serialization(Uuid, JsonError),
    #[error(transparent)]
    Load(#[from] OfferExportRepositoryError),
    #[error(transparent)]
    Record(#[from] OutboxRecordingError),
}

pub(super) struct Listener<L, R> {
    exports: L,
    outbox: R,
}

impl<L, R> Listener<L, R> {
    pub fn new(exports: L, outbox: R) -> Self {
        // Inject each storage responsibility independently.

        Self { exports, outbox }
    }
}

impl<L: OfferExportRepository, R: OutboxRepository> Listen for Listener<L, R> {
    type Event = OfferExportWasPrepared;
    type Error = ListenerError;
    const NAME: &'static str = "RecordOfferExportMessages";

    fn listen(&self, event: Self::Event, context: &EventContext) -> Result<(), Self::Error> {
        // Identify the prepared export referenced by this event.
        let export_id = event.export_id;

        // Load the completed immutable snapshot; fail if it is missing or still pending.
        let export = self
            .exports
            .load(export_id)?
            .ok_or(ListenerError::NotCompleted(export_id))?;

        // Validate the snapshot and confirm it matches the event's identity and source.
        if !export.valid_schema() || export.id != export_id || export.source != event.source {
            // Reject inconsistent snapshots before touching the outbox.

            return Err(ListenerError::Mismatch(export_id));
        }

        // Serialize one delivery message per entry, preserving its ID across retries.
        let messages = export
            .entries
            .iter()
            .map(|entry| {
                let message = OutgoingMessage {
                    id: entry.id,
                    payload: to_string(&entry.payload)
                        .map_err(|error| ListenerError::Serialization(export_id, error))?,
                    routing_key: ROUTING_KEY,
                    event_type: OfferExportWasPrepared::NAME,
                };

                Ok(message)
            })
            .collect::<Result<Vec<_>, ListenerError>>()?;

        // Persist the messages in the outbox with the triggering event's context.
        self.outbox.bulk_insert(&messages, context)?;

        // Report how many entries were queued and how many variations were excluded.
        info!(
            %export_id,
            offer_id = %export.source.offer_id,
            recorded = messages.len(),
            excluded = export.exclusions.len(),
            "offer export recorded for delivery"
        );

        // Mark this listener invocation successful after the messages have been recorded.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use bodul_shared::retailer::RetailerCode;
    use money::Money;
    use retailer_sauce::localization::TargetMarket;
    use serde_json::{Value, from_str, from_value, json};

    use super::*;
    use crate::offer_discovery::prelude::OfferExport;
    use crate::offer_discovery::prelude::export_testing::{
        ContentNextGeneration, ExclusionReason, ExportEntry, ExportExclusion, OfferImport, source,
    };
    use crate::platform::outbox::RecordError;

    #[derive(Default)]
    struct FakeExportRepository {
        export: Option<OfferExport>,
        fail_load: bool,
        requested: Mutex<Vec<Uuid>>,
    }

    impl OfferExportRepository for FakeExportRepository {
        fn load(&self, id: Uuid) -> Result<Option<OfferExport>, OfferExportRepositoryError> {
            self.requested.lock().unwrap().push(id);
            if self.fail_load {
                // Model an export-reader failure before any message recording.

                return Err(OfferExportRepositoryError::DuplicateId(id));
            }

            Ok(self.export.clone())
        }
    }

    #[derive(Default)]
    struct FakeOutboxRepository {
        fail_record: bool,
        calls: Mutex<Vec<(Vec<OutgoingMessage>, EventContext)>>,
    }

    impl OutboxRepository for FakeOutboxRepository {
        fn bulk_insert(
            &self,
            messages: &[OutgoingMessage],
            context: &EventContext,
        ) -> Result<(), OutboxRecordingError> {
            self.calls
                .lock()
                .unwrap()
                .push((messages.to_vec(), context.clone()));
            if self.fail_record {
                // Retain the attempted batch so tests can inspect the failed call.

                return Err(RecordError::Conflict(messages[0].id).into());
            }

            Ok(())
        }
    }

    fn entry(variation_index: usize, amount: i64) -> ExportEntry {
        ExportEntry {
            id: Uuid::now_v7(),
            variation_index,
            payload: OfferImport {
                was_price: Default::default(),
                availability: Default::default(),
                item_condition: Default::default(),
                content: ContentNextGeneration {
                    title: "USB charger".into(),
                    description: None,
                },
                identifiers: Default::default(),
                target_market: TargetMarket::Country {
                    code: from_value(json!("AU")).unwrap(),
                },
                language: "en".into(),
                scraped_at: "2026-10-26T12:00:00.123456Z".parse().unwrap(),
                images: vec![],
                retailer_code: RetailerCode::AnkerAu,
                price: from_value::<Money>(json!({"amount": amount, "currency": "AUD"})).unwrap(),
                url: format!("https://shop.example/Charger?variant={variation_index}#details"),
            },
        }
    }

    fn export(entries: Vec<ExportEntry>, exclusions: Vec<ExportExclusion>) -> OfferExport {
        OfferExport {
            id: Uuid::now_v7(),
            source: source(),
            entries,
            exclusions,
        }
    }

    fn event(export: &OfferExport) -> OfferExportWasPrepared {
        OfferExportWasPrepared {
            export_id: export.id,
            source: export.source.clone(),
        }
    }

    fn context() -> EventContext {
        EventContext {
            event_id: Uuid::now_v7(),
            correlation_id: Some(Uuid::now_v7()),
            causation_id: Some(Uuid::now_v7()),
            source: Some("event:OfferWasStructuralized".into()),
        }
    }

    fn build_listener(export: OfferExport) -> Listener<FakeExportRepository, FakeOutboxRepository> {
        // Successful fixtures use independently configurable stores.

        Listener::new(
            FakeExportRepository {
                export: Some(export),
                ..Default::default()
            },
            FakeOutboxRepository::default(),
        )
    }

    #[test]
    fn records_one_message_per_entry_and_none_for_exclusions() {
        let exclusion = ExportExclusion {
            variation_index: 1,
            url: "https://shop.example/Charger?variant=1".into(),
            reason: ExclusionReason::MissingPrice,
        };
        for (entries, exclusions) in [
            (vec![], vec![]),
            (vec![], vec![exclusion.clone()]),
            (vec![entry(0, 1999)], vec![]),
            (vec![entry(0, 1999), entry(2, 0)], vec![exclusion]),
        ] {
            let export = export(entries, exclusions);
            let listener = build_listener(export.clone());
            let context = context();

            listener.listen(event(&export), &context).unwrap();

            assert_eq!(*listener.exports.requested.lock().unwrap(), vec![export.id]);
            let recorded = listener.outbox.calls.lock().unwrap();
            assert_eq!(recorded.len(), 1);
            let (messages, recorded_context) = &recorded[0];
            assert_eq!(recorded_context, &context);
            assert_eq!(messages.len(), export.entries.len());
            for (message, entry) in messages.iter().zip(&export.entries) {
                assert_eq!(message.id, entry.id);
                assert_eq!(message.routing_key, "offers");
                assert_eq!(message.event_type, "OfferExportWasPrepared");
                assert_eq!(
                    from_str::<OfferImport>(&message.payload).unwrap(),
                    entry.payload
                );
            }
        }
    }

    #[test]
    fn payload_is_the_bare_import_command() {
        let export = export(vec![entry(0, 1999)], vec![]);
        let listener = build_listener(export.clone());

        listener.listen(event(&export), &context()).unwrap();

        let recorded = listener.outbox.calls.lock().unwrap();
        let payload: Value = from_str(&recorded[0].0[0].payload).unwrap();
        assert_eq!(
            payload,
            json!({
                "images": [],
                "was_price": null,
                "availability": null,
                "item_condition": null,
                "identifiers": {"gtin":null,"sku":null,"shopify_product_gid":null,"shopify_product_variation_gid":null,"shopify_handle":null},
                "target_market": {"kind":"country","code":"AU"}, "language": "en",
                "scraped_at": "2026-10-26T12:00:00.123456Z",
                "retailer_code": "AnkerAu",
                "content": {"title":"USB charger", "description":null},
                "price": {"amount": 1999, "currency": "AUD"},
                "url": "https://shop.example/Charger?variant=0#details"
            })
        );
    }

    #[test]
    fn a_missing_or_pending_export_fails() {
        let export = export(vec![entry(0, 1999)], vec![]);
        let listener = Listener::new(
            FakeExportRepository::default(),
            FakeOutboxRepository::default(),
        );

        let error = listener.listen(event(&export), &context()).unwrap_err();

        assert!(matches!(error, ListenerError::NotCompleted(id) if id == export.id));
        assert!(listener.outbox.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn load_failure_fails_the_event_without_calling_the_outbox() {
        let export = export(vec![entry(0, 1999)], vec![]);
        let listener = Listener::new(
            FakeExportRepository {
                export: Some(export.clone()),
                fail_load: true,
                ..Default::default()
            },
            FakeOutboxRepository::default(),
        );

        let error = listener.listen(event(&export), &context()).unwrap_err();

        assert!(matches!(
            error,
            ListenerError::Load(OfferExportRepositoryError::DuplicateId(id)) if id == export.id
        ));
        assert_eq!(*listener.exports.requested.lock().unwrap(), vec![export.id]);
        assert!(listener.outbox.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn recording_failure_fails_the_event_after_loading_the_export() {
        let export = export(vec![entry(0, 1999), entry(1, 2999)], vec![]);
        let listener = Listener::new(
            FakeExportRepository {
                export: Some(export.clone()),
                ..Default::default()
            },
            FakeOutboxRepository {
                fail_record: true,
                ..Default::default()
            },
        );
        let context = context();

        let error = listener.listen(event(&export), &context).unwrap_err();

        assert!(matches!(
            error,
            ListenerError::Record(OutboxRecordingError::Record(RecordError::Conflict(id)))
                if id == export.entries[0].id
        ));
        assert_eq!(*listener.exports.requested.lock().unwrap(), vec![export.id]);
        let calls = listener.outbox.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0.len(), export.entries.len());
        assert_eq!(calls[0].1, context);
    }

    #[test]
    fn a_different_snapshot_or_source_records_nothing() {
        let export = export(vec![entry(0, 1999)], vec![]);
        let mut other_source = event(&export);
        other_source.source.offer_id = Uuid::now_v7();
        let mut other_snapshot = export.clone();
        other_snapshot.entries[0].payload.language = "".into();

        let listener = build_listener(export.clone());
        assert!(matches!(
            listener.listen(other_source, &context()).unwrap_err(),
            ListenerError::Mismatch(_)
        ));
        let mismatched = build_listener(other_snapshot);
        assert!(matches!(
            mismatched.listen(event(&export), &context()).unwrap_err(),
            ListenerError::Mismatch(_)
        ));
        assert!(listener.outbox.calls.lock().unwrap().is_empty());
        assert!(mismatched.outbox.calls.lock().unwrap().is_empty());
    }
}
