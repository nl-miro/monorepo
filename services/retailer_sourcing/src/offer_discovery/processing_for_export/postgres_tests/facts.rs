use super::*;
use crate::offer_discovery::prelude::export_testing::{assert_fact_payloads, fact_document};
use crate::platform::object_storage::ObjectStorageMetadata;

fn source_with_facts(objects: &ObjectStorageRepository) -> ExportSource {
    super::super::prelude::testing::stored_source(objects, &fact_document().to_string())
}

fn row(db: &Database, id: Uuid) -> serde_json::Value {
    rows(db)
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == json!(id))
        .unwrap()
        .clone()
}

fn assert_uploaded_body(
    objects: &ObjectStorageRepository,
    db: &Database,
    export: &OfferExport,
    body: &str,
) {
    let record = row(db, export.id);
    assert_eq!(record["status"], "completed");
    assert!(record["prepared_body"].is_null());
    let storage: ObjectStorageMetadata = serde_json::from_value(record["storage"].clone()).unwrap();
    assert_eq!(storage.content_size, body.len() as u64);
    assert_eq!(
        objects
            .get(OFFER_EXPORT_BUCKET, export.id, &storage)
            .unwrap(),
        body
    );
}

/// Also runs against the configured real object-storage service.
pub(super) fn verify_current_round_trip(db: &Database, objects: ObjectStorageRepository) {
    let source = source_with_facts(&objects);
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let prepared = store.prepare(source.clone()).unwrap();
    let expected = prepared.snapshot.clone();
    let body = prepared.body.clone();
    assert_fact_payloads(&expected, &fact_document());
    assert_eq!(expected.source, source);
    assert_eq!(row(db, expected.id)["prepared_body"], body);
    assert!(store.load(expected.id).unwrap().is_none());
    drop(store);
    let restarted = PgStore::new(db.pool.clone(), objects.clone());
    let retry = restarted.prepare(source.clone()).unwrap();
    assert_eq!(retry.body, body);
    assert_eq!(retry.snapshot, expected);
    assert_eq!(restarted.complete(retry).unwrap(), expected);
    assert_uploaded_body(&objects, db, &expected, &body);
    drop(restarted);
    let restarted = PgStore::new(db.pool.clone(), objects);
    assert_eq!(restarted.load(expected.id).unwrap(), Some(expected.clone()));
    let retry = restarted.prepare(source).unwrap();
    assert_eq!(retry.body, body);
    assert_eq!(retry.snapshot, expected);
    assert!(retry.storage.is_some());
    assert_eq!(restarted.complete(retry).unwrap(), expected);
    assert_eq!(rows(db).as_array().unwrap().len(), 1);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn current_facts_round_trip_preserves_exact_values_and_bytes() {
    let server = Server::new();
    verify_current_round_trip(&Database::new(), server.client());
    let data = server.data.lock().unwrap();
    assert_eq!(
        data.uploads
            .iter()
            .filter(|(bucket, _)| bucket == OFFER_EXPORT_BUCKET)
            .count(),
        1
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn facts_survive_pending_and_completed_retries() {
    let db = Database::new();
    let server = Server::new();
    let objects = server.client();
    let source = source_with_facts(&objects);
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let prepared = store.prepare(source.clone()).unwrap();
    let expected = prepared.snapshot.clone();
    let body = prepared.body.clone();
    assert_fact_payloads(&expected, &fact_document());
    let restarted = PgStore::new(db.pool.clone(), objects.clone());
    let retry = restarted.prepare(source.clone()).unwrap();
    assert_eq!(retry.body, body);
    assert_eq!(retry.snapshot, expected);
    assert_eq!(restarted.complete(retry).unwrap(), expected);
    assert_uploaded_body(&objects, &db, &expected, &body);
    let completed_retry = restarted.prepare(source).unwrap();
    assert_eq!(completed_retry.body, body);
    assert_eq!(completed_retry.snapshot, expected);
    assert_eq!(rows(&db).as_array().unwrap().len(), 1);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn facts_recover_identical_bytes_after_upload_then_completion_failure() {
    let db = Database::new();
    let server = Server::new();
    let source = source_with_facts(&server.client());
    let store = PgStore::new(db.pool.clone(), server.client());
    let prepared = store.prepare(source.clone()).unwrap();
    let expected = prepared.snapshot.clone();
    let body = prepared.body.clone();
    let pending = row(&db, expected.id);
    db.pool.get().unwrap().batch_execute("ALTER TABLE offer_export ADD CONSTRAINT fail_fact_completion CHECK (status <> 'completed') NOT VALID").unwrap();
    assert!(store.complete(prepared).is_err());
    assert_eq!(row(&db, expected.id), pending);
    assert!(store.load(expected.id).unwrap().is_none());
    db.pool
        .get()
        .unwrap()
        .batch_execute("ALTER TABLE offer_export DROP CONSTRAINT fail_fact_completion")
        .unwrap();
    drop(store);
    // Recovery must use durable prepared bytes, even if the original input is unavailable.
    server
        .data
        .lock()
        .unwrap()
        .records
        .remove(&source.storage.object_storage_id)
        .unwrap();
    let restarted = PgStore::new(db.pool.clone(), server.client());
    let retry = restarted.prepare(source.clone()).unwrap();
    assert_eq!(retry.body, body);
    assert_eq!(retry.snapshot, expected);
    assert_fact_payloads(&retry.snapshot, &fact_document());
    assert_eq!(restarted.complete(retry).unwrap(), expected);
    assert_uploaded_body(&server.client(), &db, &expected, &body);
    assert_eq!(restarted.prepare(source).unwrap().body, body);
    let data = server.data.lock().unwrap();
    let uploads: Vec<_> = data
        .uploads
        .iter()
        .filter(|(bucket, request)| {
            bucket == OFFER_EXPORT_BUCKET && request["idempotency_key"] == json!(expected.id)
        })
        .collect();
    assert_eq!(uploads.len(), 2);
    assert_eq!(uploads[0], uploads[1]);
    assert_eq!(uploads[0].1["payload"], body);
    assert_eq!(
        data.records
            .values()
            .filter(|record| record["bucket"] == OFFER_EXPORT_BUCKET
                && record["idempotency_key"] == json!(expected.id))
            .count(),
        1
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn malformed_observations_never_persist_an_export_preparation() {
    let db = Database::new();
    let server = Server::new();
    let store = PgStore::new(db.pool.clone(), server.client());
    for (field, value) in [
        ("was_price", json!({"amount":1,"currency":"JPY"})),
        (
            "was_price",
            json!({"amount":9_007_199_254_740_994_i64,"currency":"USD"}),
        ),
        ("availability", json!({"other":" "})),
        ("item_condition", json!("guessed_new")),
        ("content", json!(null)),
        ("content", json!({})),
        ("content", json!({"description":"bad\u{0}"})),
    ] {
        let mut document = fact_document();
        document["variations"][0][field] = value;
        let source =
            super::super::prelude::testing::stored_source(&server.client(), &document.to_string());
        assert!(store.prepare(source).is_err());
        assert_eq!(rows(&db), json!([]));
    }
    assert!(
        server
            .data
            .lock()
            .unwrap()
            .uploads
            .iter()
            .all(|(bucket, _)| bucket != OFFER_EXPORT_BUCKET)
    );
}
