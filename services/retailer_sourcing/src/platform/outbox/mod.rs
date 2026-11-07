//! Records outgoing messages in Mulac's outbox for the publisher to deliver.
//!
//! A message's ID is its row ID and delivery identity. Recording is
//! idempotent: an existing row is never changed, so a repeated recording keeps
//! the first metadata, status, attempts and reservation. Reusing an ID for
//! different content fails instead.

#![cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "legacy recorder retained while repository stages are stubbed"
    )
)]

pub(crate) mod error;
pub(crate) mod model;
pub(crate) mod record;
pub(crate) mod repository;
pub(crate) mod schema;

use diesel::prelude::*;
use diesel::sql_types::{Array, Integer, Jsonb, Text, Uuid as SqlUuid};
use outbox::prelude::{OutboxEntryMetadata, OutboxStatus};
use uuid::Uuid;

use super::bus::EventContext;
use super::db;

const CONTENT_TYPE: &str = "application/json";
/// One message to record, with the event that caused it.
#[derive(Debug, Clone, PartialEq)]
pub struct OutgoingMessage {
    /// Stable delivery identity, used as both row and message ID.
    pub id: Uuid,
    /// A JSON payload, stored as this exact text.
    pub payload: String,
    pub routing_key: &'static str,
    pub event_type: &'static str,
}

impl OutgoingMessage {
    /// Combine this message's delivery identity with the event that caused it.
    pub fn metadata(&self, context: &EventContext) -> OutboxEntryMetadata {
        OutboxEntryMetadata {
            sequence: None,
            event_id: context.event_id,
            message_id: self.id,
            correlation_id: context.correlation_id,
            causation_id: context.causation_id,
            event_type: self.event_type.to_string(),
            routing_key: self.routing_key.to_string(),
            source: context.source.clone(),
            content_type: Some(CONTENT_TYPE.to_string()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RecordError {
    #[error("outbox metadata serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),
    #[error("outbox message {0} payload is invalid")]
    InvalidPayload(Uuid),
    #[error("outbox message {0} is listed more than once")]
    Duplicate(Uuid),
    #[error("outbox message {0} already exists with different content")]
    Conflict(Uuid),
    #[error("outbox messages were not recorded: {0:?}")]
    Missing(Vec<Uuid>),
}

#[derive(diesel::QueryableByName)]
struct ExistingRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    payload: String,
    #[diesel(sql_type = Jsonb)]
    meta: serde_json::Value,
}

/// Records every message in one transaction, or none of them.
pub fn record(
    conn: &mut PgConnection,
    messages: &[OutgoingMessage],
    context: &EventContext,
) -> Result<(), RecordError> {
    // An empty batch succeeds without opening a transaction.
    if messages.is_empty() {
        return Ok(());
    }

    // Sort and validate message identities before parsing their payloads.
    let messages = grab_unique_messages(messages)?;
    let payloads = messages
        .iter()
        .map(|message| parse_object(message.id, &message.payload))
        .collect::<Result<Vec<_>, _>>()?;

    // Prepare aligned arrays of IDs and event metadata for the batch insert.
    let ids: Vec<Uuid> = messages.iter().map(|message| message.id).collect();
    let metas = messages
        .iter()
        .map(|message| serde_json::to_value(message.metadata(context)))
        .collect::<Result<Vec<_>, _>>()?;

    // Roll back new rows if insertion or any subsequent replay check fails.

    conn.transaction(|conn| {
        // Insert the full batch as received, preserving existing rows and their delivery state.
        // PostgreSQL unnest combines corresponding ID, payload and metadata array elements.
        let sql = r#"
            INSERT INTO outbox_entries ( id, status, payload, meta )
            (
                SELECT id, $2, payload, meta
                FROM UNNEST ( $1, $3, $4 ) AS m(id, payload, meta)
                ORDER BY id
            )
            ON CONFLICT (id) DO NOTHING
        "#;

        diesel::sql_query(sql)
            .bind::<Array<SqlUuid>, _>(&ids)
            .bind::<Integer, _>(i32::from(OutboxStatus::Received))
            .bind::<Array<Text>, _>(
                messages
                    .iter()
                    .map(|m| m.payload.as_str())
                    .collect::<Vec<_>>(),
            )
            .bind::<Array<Jsonb>, _>(&metas)
            .execute(conn)?;

        check_insert_results(conn, &ids, &messages, &payloads)
    })
}

/// Read and validate the batch within the insert transaction so failures roll back new rows.
fn check_insert_results(
    conn: &mut PgConnection,
    ids: &[Uuid],
    messages: &[&OutgoingMessage],
    payloads: &[serde_json::Value],
) -> Result<(), RecordError> {
    // Read both newly inserted and pre-existing messages in the same order as the batch.
    let sql = "SELECT id, payload, meta FROM outbox_entries WHERE id = ANY($1) ORDER BY id";
    let existing: Vec<ExistingRow> = diesel::sql_query(sql)
        .bind::<Array<SqlUuid>, _>(ids)
        .load(conn)?;

    // Report every missing ID before comparing rows, keeping messages and rows aligned.
    verify_recorded_ids(ids, &existing)?;

    // Verify delivery content; repeated notifications may have different event metadata.
    for ((message, payload), row) in messages.iter().zip(payloads).zip(existing.iter()) {
        if !same_content(message, payload, row) {
            return Err(RecordError::Conflict(message.id));
        }
    }

    Ok(())
}

/// Sort by ID for consistent database locking and reject duplicate delivery identities.
pub(crate) fn grab_unique_messages(
    messages: &[OutgoingMessage],
) -> Result<Vec<&OutgoingMessage>, RecordError> {
    let mut messages: Vec<&OutgoingMessage> = messages.iter().collect();
    messages.sort_by_key(|message| message.id);
    for pair in messages.windows(2) {
        if pair[0].id == pair[1].id {
            // Duplicate IDs are invalid even when their payloads match.

            return Err(RecordError::Duplicate(pair[0].id));
        }
    }

    Ok(messages)
}

/// The rows must be sorted by ID, as returned by the recording query.
fn verify_recorded_ids(ids: &[Uuid], existing: &[ExistingRow]) -> Result<(), RecordError> {
    let missing: Vec<Uuid> = ids
        .iter()
        .copied()
        .filter(|id| existing.binary_search_by_key(id, |row| row.id).is_err())
        .collect();
    if !missing.is_empty() {
        return Err(RecordError::Missing(missing));
    }

    Ok(())
}

/// Validate JSON syntax without retaining the parsed value.
pub(crate) fn validate_payload(id: Uuid, payload: &str) -> Result<(), RecordError> {
    serde_json::from_str::<serde_json::Value>(payload)
        .map_err(|_| RecordError::InvalidPayload(id))?;

    Ok(())
}

fn parse_object(id: Uuid, payload: &str) -> Result<serde_json::Value, RecordError> {
    match serde_json::from_str(payload) {
        Ok(value @ serde_json::Value::Object(_)) => Ok(value),
        _ => Err(RecordError::InvalidPayload(id)),
    }
}

/// The delivery content must match; the causing event's metadata may differ,
/// since a repeated notification keeps the first one recorded.
fn same_content(message: &OutgoingMessage, payload: &serde_json::Value, row: &ExistingRow) -> bool {
    let Ok(meta) = serde_json::from_value::<OutboxEntryMetadata>(row.meta.clone()) else {
        return false;
    };
    serde_json::from_str::<serde_json::Value>(&row.payload)
        .ok()
        .as_ref()
        == Some(payload)
        && meta.message_id == message.id
        && meta.routing_key == message.routing_key
        && meta.event_type == message.event_type
        && meta.content_type.as_deref() == Some(CONTENT_TYPE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(payload: &str) -> OutgoingMessage {
        OutgoingMessage {
            id: Uuid::now_v7(),
            payload: payload.to_string(),
            routing_key: "offers",
            event_type: "OfferExportWasPrepared",
        }
    }

    #[test]
    fn reports_all_missing_ids_including_gaps_and_trailing_rows() {
        let ids: Vec<Uuid> = (1..=7).map(Uuid::from_u128).collect();
        for present in [vec![], vec![1, 3, 4], (0..7).collect()] {
            let rows: Vec<ExistingRow> = present
                .iter()
                .map(|&index| ExistingRow {
                    id: ids[index],
                    payload: "{}".into(),
                    meta: serde_json::Value::Null,
                })
                .collect();
            let result = verify_recorded_ids(&ids, &rows);
            if present.len() == ids.len() {
                assert!(result.is_ok());
            } else {
                let expected = if present.is_empty() {
                    ids.clone()
                } else {
                    vec![ids[0], ids[2], ids[5], ids[6]]
                };
                let error = result.unwrap_err();
                assert!(matches!(&error, RecordError::Missing(missing) if *missing == expected));
                for id in expected {
                    assert!(error.to_string().contains(&id.to_string()));
                }
            }
        }
        assert!(verify_recorded_ids(&[], &[]).is_ok());
    }

    #[test]
    fn metadata_combines_the_message_and_its_causing_event() {
        let message = message("{}");
        let context = EventContext {
            event_id: Uuid::now_v7(),
            correlation_id: Some(Uuid::now_v7()),
            causation_id: None,
            source: Some("event:OfferExportWasPrepared".into()),
        };

        assert_eq!(
            message.metadata(&context),
            OutboxEntryMetadata {
                sequence: None,
                event_id: context.event_id,
                message_id: message.id,
                correlation_id: context.correlation_id,
                causation_id: None,
                event_type: "OfferExportWasPrepared".into(),
                routing_key: "offers".into(),
                source: context.source.clone(),
                content_type: Some("application/json".into()),
            }
        );
    }

    #[test]
    fn payloads_must_be_json_objects() {
        for payload in ["[]", "1", "not json", ""] {
            let message = message(payload);
            assert!(matches!(
                parse_object(message.id, &message.payload),
                Err(RecordError::InvalidPayload(id)) if id == message.id
            ));
        }
        assert!(parse_object(Uuid::now_v7(), r#"{"a":1}"#).is_ok());
    }

    #[test]
    fn content_comparison_ignores_event_metadata_but_not_delivery_content() {
        let message = message(r#"{"title":"a","price":{"amount":1}}"#);
        let payload = parse_object(message.id, &message.payload).unwrap();
        let other_event = EventContext {
            event_id: Uuid::now_v7(),
            correlation_id: None,
            causation_id: None,
            source: None,
        };
        let row = |payload: &str, meta: OutboxEntryMetadata| ExistingRow {
            id: message.id,
            payload: payload.to_string(),
            meta: serde_json::to_value(meta).unwrap(),
        };
        let meta = message.metadata(&other_event);

        assert!(same_content(
            &message,
            &payload,
            &row(r#"{"price":{"amount":1}, "title":"a"}"#, meta.clone())
        ));
        assert!(!same_content(
            &message,
            &payload,
            &row(r#"{"title":"b","price":{"amount":1}}"#, meta.clone())
        ));
        let mut rerouted = meta.clone();
        rerouted.routing_key = "elsewhere".into();
        assert!(!same_content(
            &message,
            &payload,
            &row(&message.payload, rerouted)
        ));
        let mut renamed = meta;
        renamed.message_id = Uuid::now_v7();
        assert!(!same_content(
            &message,
            &payload,
            &row(&message.payload, renamed)
        ));
    }
}
