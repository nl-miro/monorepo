use super::command::ProcessOfferForExport;
use super::entity::{ExportSource, OfferExport, PreparedExport};
use super::event::OfferExportWasPrepared;
use super::store::StoreError;
use crate::platform::bus::Handle;

pub(super) trait Store: Send + Sync + 'static {
    fn prepare(&self, source: ExportSource) -> Result<PreparedExport, StoreError>;
    fn complete(&self, prepared: PreparedExport) -> Result<OfferExport, StoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("scrape time disagrees with the pinned structuralized artifact")]
    ConflictingScrapeTime,
    #[error(
        "offer {0} has no pinned structuralized source; run fresh structuralization before exporting"
    )]
    UnpinnedSource(uuid::Uuid),
    #[error(transparent)]
    Store(#[from] StoreError),
}

pub(super) struct Handler<S> {
    store: S,
}

impl<S> Handler<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> Handle for Handler<S> {
    type Command = ProcessOfferForExport;
    type Event = OfferExportWasPrepared;
    type Error = HandlerError;

    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        let storage = command
            .storage
            .ok_or(HandlerError::UnpinnedSource(command.offer_id))?;
        if command.scraped_at != storage.scraped_at {
            return Err(HandlerError::ConflictingScrapeTime);
        }
        let source = ExportSource {
            offer_id: command.offer_id,
            consolidated_sitemap_id: command.consolidated_sitemap_id,
            retailer_code: command.retailer_code,
            url: command.url,
            storage,
        };
        let prepared = self.store.prepare(source)?;
        let snapshot = self.store.complete(prepared)?;
        tracing::info!(
            export_id = %snapshot.id,
            offer_id = %snapshot.source.offer_id,
            exported = snapshot.entries.len(),
            excluded = snapshot.exclusions.len(),
            "offer processed for export"
        );
        Ok(vec![OfferExportWasPrepared {
            export_id: snapshot.id,
            source: snapshot.source,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::super::conversion::{
        convert,
        tests::{ANKER, source},
    };
    use super::*;
    use crate::offer_discovery::structuralize_offer::prelude::OfferWasStructuralized;

    struct FakeStore {
        fail_prepare: bool,
        fail_complete: bool,
    }

    impl Store for FakeStore {
        fn prepare(&self, source: ExportSource) -> Result<PreparedExport, StoreError> {
            if self.fail_prepare {
                return Err(StoreError::Incomplete);
            }
            let snapshot = convert(source, ANKER)?;
            Ok(PreparedExport {
                body: serde_json::to_string(&snapshot)?,
                snapshot,
                storage: None,
            })
        }
        fn complete(&self, prepared: PreparedExport) -> Result<OfferExport, StoreError> {
            if self.fail_complete {
                return Err(StoreError::Incomplete);
            }
            Ok(prepared.snapshot)
        }
    }

    fn command() -> ProcessOfferForExport {
        let source = source();
        ProcessOfferForExport {
            scraped_at: source.storage.scraped_at,
            offer_id: source.offer_id,
            consolidated_sitemap_id: source.consolidated_sitemap_id,
            retailer_code: source.retailer_code,
            url: source.url,
            storage: Some(source.storage),
        }
    }

    #[test]
    fn export_messages_have_no_schema_version() {
        let command = command();
        let snapshot = convert(source(), ANKER).unwrap();
        let event = OfferExportWasPrepared {
            export_id: snapshot.id,
            source: snapshot.source.clone(),
        };
        for value in [
            serde_json::to_value(command).unwrap(),
            serde_json::to_value(snapshot).unwrap(),
            serde_json::to_value(event).unwrap(),
        ] {
            assert!(value.get("schema_version").is_none());
        }
    }

    #[test]
    fn only_successful_completion_returns_an_export_event() {
        for (fail_prepare, fail_complete) in [(true, false), (false, true), (false, false)] {
            let handler = Handler::new(FakeStore {
                fail_prepare,
                fail_complete,
            });
            let command = command();
            let result = handler.handle(command.clone());
            if fail_prepare || fail_complete {
                assert!(result.is_err());
            } else {
                let events = result.unwrap();
                assert_eq!(events.len(), 1);
                assert_eq!(events[0].source.offer_id, command.offer_id);
                assert_eq!(events[0].source.storage, command.storage.unwrap());
            }
        }
    }

    #[test]
    fn export_rejects_a_timestamp_from_another_source_before_preparing() {
        let handler = Handler::new(FakeStore {
            fail_prepare: true,
            fail_complete: true,
        });
        let mut command = command();
        command.scraped_at = Some("2026-10-26T12:00:00Z".parse().unwrap());
        assert!(matches!(
            handler.handle(command),
            Err(HandlerError::ConflictingScrapeTime)
        ));
    }

    #[test]
    fn legacy_event_decodes_but_requires_fresh_structuralization() {
        let mut json = serde_json::to_value(command()).unwrap();
        json.as_object_mut().unwrap().remove("storage");
        let defaulted: OfferWasStructuralized = serde_json::from_value(json).unwrap();
        assert!(defaulted.storage.is_none());
        let command = super::super::reacts_to::on(&defaulted).unwrap();
        let handler = Handler::new(FakeStore {
            fail_prepare: true,
            fail_complete: true,
        });
        let error = handler.handle(command).unwrap_err();
        assert!(matches!(error, HandlerError::UnpinnedSource(id) if id == defaulted.offer_id));
        assert!(error.to_string().contains("fresh structuralization"));
    }
}
