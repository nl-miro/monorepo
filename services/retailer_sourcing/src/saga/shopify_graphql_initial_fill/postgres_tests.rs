use super::listener::Listener;
use super::register;
use crate::app;
use crate::platform::bus::{Bus, Event, EventEnvelope, IntoEventCommand};
use crate::platform::state::AppState;
use crate::platform::test_db::{Database, json_query};
use crate::saga::saga_repository::SagaRepository;
use crate::steps::prelude::RetailerFillInitialized;
use axum::{body::Body, http::Request};
use diesel::sql_types::{Jsonb, Text, Uuid as SqlUuid};
use diesel::{RunQueryDsl, sql_query};
use http_body_util::BodyExt;
use kernel::NewEventMetadata;
use serde_json::{Value, from_slice, json, to_string};
use shared::retailer::RetailerCode;
use tower::ServiceExt;
use uuid::Uuid;

fn envelope(
    retailer_code: RetailerCode,
    correlation: Uuid,
) -> EventEnvelope<RetailerFillInitialized> {
    EventEnvelope {
        event: RetailerFillInitialized { retailer_code },
        metadata: Some(NewEventMetadata {
            event_id: Uuid::now_v7(),
            correlation_id: Some(correlation),
            causation_id: Some(Uuid::now_v7()),
            source: Some("test".into()),
            sequence: None,
        }),
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn initialization_replays_snapshot_and_validates_correlation_and_retailer() {
    let db = Database::new();
    let listener = Listener::new(SagaRepository::new(db.pool.clone()));
    let correlation = Uuid::now_v7();
    let event = envelope(RetailerCode::UgreenEu, correlation);
    let first = listener.on_retailer_fill_initialized(&event).unwrap();
    let first: Vec<_> = first
        .into_iter()
        .map(|command| command.encode().unwrap())
        .collect();
    assert!(first.len() > 1);
    assert_ne!(first[0].id, first[1].id);
    let mut replay = envelope(RetailerCode::UgreenEu, correlation);
    let second = listener.on_retailer_fill_initialized(&replay).unwrap();
    for (first, second) in first.iter().zip(second) {
        let second = second.encode().unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(first.command.payload, second.command.payload);
    }
    let row = json_query(
        &mut db.pool.get().unwrap(),
        "SELECT to_jsonb(s) AS value FROM sagas s",
    );
    assert_ne!(row["id"], json!(correlation));
    assert_eq!(row["correlation_id"], json!(correlation));
    assert_eq!(row["status"], "in_progress");
    assert!(row["payload"].get("saga_id").is_none());
    assert!(row["payload"].get("retailer_code").is_none());
    replay.event.retailer_code = RetailerCode::AnkerAu;
    assert!(listener.on_retailer_fill_initialized(&replay).is_err());
    replay.metadata = None;
    assert!(listener.on_retailer_fill_initialized(&replay).is_err());
    assert!(
        listener
            .on_retailer_fill_initialized(&envelope(RetailerCode::AnkerAu, Uuid::nil()))
            .is_err()
    );
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn registered_initialization_creates_one_saga_and_deduplicates_dispatch() {
    let db = Database::new();
    let correlation = Uuid::now_v7();
    for _ in 0..2 {
        let event = envelope(RetailerCode::AnkerAu, correlation);
        let meta = event.metadata.unwrap();
        sql_query("INSERT INTO event_entries (id, event_type, payload, meta, status) VALUES ($1,$2,$3,$4,0)")
            .bind::<SqlUuid, _>(meta.event_id).bind::<Text, _>(RetailerFillInitialized::NAME)
            .bind::<Text, _>(to_string(&event.event).unwrap()).bind::<Jsonb, _>(json!(meta))
            .execute(&mut db.pool.get().unwrap()).unwrap();
    }
    let mut bus = Bus::default();
    register(&mut bus, &db.pool);
    let router = app(AppState {
        bus: Some(bus.start(db.pool.clone()).unwrap()),
        ..Default::default()
    });
    let request = Request::post("/api/events/run")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"limit":10}"#))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert!(response.status().is_success());
    let result: Value =
        from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(result["errors"], json!([]));
    assert_eq!(result["processed"], 2);
    let counts = json_query(
        &mut db.pool.get().unwrap(),
        "SELECT jsonb_build_object('sagas', (SELECT count(*) FROM sagas), 'commands', (SELECT count(*) FROM command_entries)) AS value",
    );
    assert_eq!(counts, json!({"sagas":1,"commands":1}));
    let command = json_query(
        &mut db.pool.get().unwrap(),
        "SELECT to_jsonb(c) AS value FROM command_entries c",
    );
    assert_eq!(command["meta"]["correlation_id"], json!(correlation));
    assert_eq!(command["command_type"], "AcquireStorefrontToken");
}
