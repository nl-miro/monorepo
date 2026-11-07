use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::PoolError;
use outbox::prelude::{NewOutboxEntry, OutboxStatus};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::platform::bus::EventContext;
use crate::platform::db::DbPool;
use crate::platform::outbox::{
    OutgoingMessage, RecordError, grab_unique_messages, validate_payload,
};

#[derive(Debug, Error)]
pub(super) enum OutboxRecordingError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),
    #[error(transparent)]
    Record(#[from] RecordError),
}

pub(super) trait OutboxRepository: Send + Sync + 'static {
    /// Record the entire batch atomically, preserving existing matching messages.
    fn bulk_insert(
        &self,
        messages: &[OutgoingMessage],
        context: &EventContext,
    ) -> Result<(), OutboxRecordingError>;
}

/// One complete row to insert into the outbox.
#[derive(Insertable)]
#[diesel(table_name = outbox_entries)]
struct OutboxEntryRecord {
    id: Uuid,
    status: i32,
    payload: String,
    meta: Value,
    scheduled_at: DateTime<Utc>,
    received_at: DateTime<Utc>,
}

impl TryFrom<NewOutboxEntry> for OutboxEntryRecord {
    type Error = RecordError;

    fn try_from(entry: NewOutboxEntry) -> Result<Self, Self::Error> {
        let meta = serde_json::to_value(entry.meta)?;

        Ok(Self {
            id: entry.id,
            status: i32::from(OutboxStatus::Received),
            payload: entry.payload,
            meta,
            scheduled_at: entry.scheduled_at,
            received_at: entry.received_at,
        })
    }
}

/// Records a complete message batch in PostgreSQL without loading exports.
pub(super) struct PgOutboxRepository {
    pool: DbPool,
}

impl PgOutboxRepository {
    pub fn new(pool: DbPool) -> Self {
        // The outbox needs only the database, independently of export loading.

        Self { pool }
    }
}

// Only the columns supplied by this insert are needed; other columns use database defaults.
diesel::table! {
    outbox_entries (id) {
        id -> Uuid,
        status -> Int4,
        payload -> Text,
        meta -> Jsonb,
        scheduled_at -> Timestamptz,
        received_at -> Timestamptz,
    }
}

impl OutboxRepository for PgOutboxRepository {
    fn bulk_insert(
        &self,
        messages: &[OutgoingMessage],
        context: &EventContext,
    ) -> Result<(), OutboxRecordingError> {
        if messages.is_empty() {
            // An empty batch requires no connection or validation.
            return Ok(());
        }

        let records = Self::generate_entries(messages, context)?;

        self.insert_entries(records)
    }
}

impl PgOutboxRepository {
    fn insert_entries(&self, input: Vec<NewOutboxEntry>) -> Result<(), OutboxRecordingError> {
        let records = input
            .into_iter()
            .map(OutboxEntryRecord::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let mut conn = self.pool.get()?;

        conn.transaction(|conn| {
            self.insert_records(conn, &records)?;

            self.check_insertion_results(conn, &records)
        })
        .map_err(OutboxRecordingError::from)
    }

    fn generate_entries(
        messages: &[OutgoingMessage],
        context: &EventContext,
    ) -> Result<Vec<NewOutboxEntry>, OutboxRecordingError> {
        let messages = grab_unique_messages(messages)?;
        let mut records = Vec::with_capacity(messages.len());
        for message in messages {
            // Validate JSON while retaining the exact original payload text for storage.
            validate_payload(message.id, &message.payload)?;

            let timestamp = Utc::now();
            let record = NewOutboxEntry {
                id: message.id,
                payload: message.payload.clone(),
                meta: message.metadata(context),
                scheduled_at: timestamp,
                received_at: timestamp,
            };

            records.push(record);
        }

        Ok(records)
    }

    fn insert_records(
        &self,
        conn: &mut PgConnection,
        records: &[OutboxEntryRecord],
    ) -> Result<(), RecordError> {
        diesel::insert_into(outbox_entries::table)
            .values(records)
            .on_conflict(outbox_entries::id)
            .do_nothing()
            .execute(conn)?;

        Ok(())
    }

    fn check_insertion_results(
        &self,
        _conn: &mut PgConnection,
        _inputs: &[OutboxEntryRecord],
    ) -> Result<(), RecordError> {
        // Verify the recorded batch when implementing this stage.

        todo!("check outbox insertion results")
    }
}

#[cfg(test)]
mod tests {
    use outbox::prelude::OutboxEntryMetadata;

    use super::*;

    fn message(id: u128, payload: &str) -> OutgoingMessage {
        let id = Uuid::from_u128(id);

        OutgoingMessage {
            id,
            payload: payload.into(),
            routing_key: "offers",
            event_type: "OfferExportWasPrepared",
        }
    }

    fn context() -> EventContext {
        let event_id = Uuid::from_u128(10);

        EventContext {
            event_id,
            correlation_id: Some(Uuid::from_u128(11)),
            causation_id: Some(Uuid::from_u128(12)),
            source: Some("export".into()),
        }
    }

    #[test]
    fn generates_ordered_records_preserving_payloads_and_event_metadata() {
        let messages = [
            message(2, "{ \"second\": 2 }"),
            message(1, "{ \"first\": 1 }"),
        ];
        let context = context();
        let inputs = PgOutboxRepository::generate_entries(&messages, &context).unwrap();
        assert_eq!(inputs.len(), 2);
        for (record, message) in inputs.iter().zip(messages.iter().rev()) {
            assert_eq!(record.id, message.id);
            assert_eq!(record.payload, message.payload);
            assert_eq!(record.meta, message.metadata(&context));
            let stored = OutboxEntryRecord::try_from(record.clone()).unwrap();
            assert_eq!(stored.id, record.id);
            assert_eq!(stored.payload, record.payload);
            assert_eq!(stored.status, i32::from(OutboxStatus::Received));
            assert_eq!(stored.scheduled_at, record.scheduled_at);
            assert_eq!(stored.received_at, record.received_at);
            let meta: OutboxEntryMetadata = serde_json::from_value(stored.meta).unwrap();
            assert_eq!(meta, record.meta);
        }
        assert!(
            PgOutboxRepository::generate_entries(&[], &context)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn rejects_duplicate_ids_even_with_matching_payloads() {
        let messages = [message(2, "{}"), message(1, "{}"), message(2, "{}")];

        assert!(matches!(
            PgOutboxRepository::generate_entries(&messages, &context()),
            Err(OutboxRecordingError::Record(RecordError::Duplicate(id))) if id == Uuid::from_u128(2)
        ));
    }

    #[test]
    fn accepts_valid_json_values_and_preserves_their_text() {
        for payload in ["[]", "null", "42", "true", "\"text\""] {
            let messages = [message(1, payload)];
            let records = PgOutboxRepository::generate_entries(&messages, &context()).unwrap();
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].payload, payload);
        }
    }

    #[test]
    fn rejects_invalid_json_payloads() {
        for payload in ["invalid", "", "{", "[1,]"] {
            let messages = [message(1, "{}"), message(2, payload)];
            assert!(matches!(
                PgOutboxRepository::generate_entries(&messages, &context()),
                Err(OutboxRecordingError::Record(RecordError::InvalidPayload(id))) if id == Uuid::from_u128(2)
            ));
        }
    }
}
