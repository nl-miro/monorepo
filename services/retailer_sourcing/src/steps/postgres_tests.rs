//! Durable end-to-end tests in an isolated schema, with local object-storage fixtures.
use super::graphql::offer_details::Store;
use super::graphql::offer_details::{OfferDetailsFetched, SourceType};
use super::graphql::product_catalog_ids as catalog;
use super::graphql::structuralize_offer_details::OfferDetailsStructuralized;
use crate::app;
use crate::platform::Platform;
use crate::platform::http::Client;
use crate::platform::{
    bus::{Bus, Event},
    state::AppState,
    test_db::{Database, json_query},
    test_objects::Server,
};
use crate::saga::prelude::{
    DieselPgSagaRepository, SagaId, SagaRepository, ShopifyGraphQlInitialFillSaga,
    register_initial_fill_listeners,
};
use crate::steps::prelude::{
    register_html_steps, register_initial_fill, register_offer_processing_steps,
    register_shopify_steps, register_sitemap_steps,
};
use axum::Router;
use axum::{
    body::{Body, Bytes, to_bytes},
    http::{Request, StatusCode},
};
use catalog::FetchProductCatalogIdentifiers;
use chrono::Utc;
use diesel::sql_query;
use diesel::{
    prelude::*,
    sql_types::{Jsonb, Text, Uuid as SqlUuid},
};
use retailer_data_extractor::get_storefront_config;
use serde_json::from_slice;
use serde_json::to_string;
use serde_json::{Value, json};
use serde_json::{from_str, from_value};
use shared::retailer::RetailerCode;
use std::collections::HashSet;
use tower::ServiceExt;
use uuid::Uuid;

fn rows(db: &Database, table: &str) -> Value {
    json_query(
        &mut db.pool.get().unwrap(),
        &format!("SELECT coalesce(jsonb_agg(to_jsonb(t)), '[]') AS value FROM {table} t"),
    )
}
fn post_bytes(server: &Server, app: &Router, path: &str, body: Value) -> (StatusCode, Bytes) {
    server.runtime.block_on(async {
        let response = app
            .clone()
            .oneshot(
                Request::post(path)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, body)
    })
}
fn post(server: &Server, app: &Router, path: &str, body: Value) -> (StatusCode, Value) {
    let (status, bytes) = post_bytes(server, app, path, body);
    (status, from_slice(&bytes).unwrap())
}
fn run(server: &Server, app: &Router, path: &str) {
    let (status, body) = post(server, app, path, json!({"limit":100}));
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["errors"], json!([]), "{body}");
}
#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_initial_fill_step_emits_one_request_per_storefront() {
    let db = Database::new();
    let server = Server::new();
    let mut bus = Bus::default();
    register_initial_fill(&mut bus);
    let guard = server.runtime.enter();
    let bus = bus.start(db.pool.clone()).unwrap();
    drop(guard);
    let app = app(AppState {
        bus: Some(bus.clone()),
        ..Default::default()
    });
    let (status, response) = post_bytes(
        &server,
        &app,
        "/api/initial-fill",
        json!(["ankerau", "ugreenkr"]),
    );
    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(response.is_empty());
    let commands = rows(&db, "command_entries");
    let commands = commands.as_array().unwrap();
    assert_eq!(commands.len(), 2);
    run(&server, &app, "/api/commands/run");
    let events = rows(&db, "event_entries");
    let events = events.as_array().unwrap();
    assert_eq!(events.len(), 2);
    let mut retailers = HashSet::new();
    let mut saga_ids = HashSet::new();
    for command in commands {
        assert_eq!(command["command_type"], "RequestInitialFill");
        let payload: Value = from_str(command["payload"].as_str().unwrap()).unwrap();
        let saga_id: SagaId = from_value(payload["saga_id"].clone()).unwrap();
        assert!(saga_ids.insert(Uuid::from(saga_id)));
        retailers.insert(payload["retailer_code"].as_str().unwrap().to_owned());
        assert!(events.iter().any(|event| {
            let fact: Value = from_str(event["payload"].as_str().unwrap()).unwrap();
            event["event_type"] == "InitialFillRequested"
                && fact["saga_id"] == payload["saga_id"]
                && fact["retailer_code"] == payload["retailer_code"]
        }));
    }
    assert_eq!(
        retailers,
        HashSet::from(["ankerau".to_owned(), "ugreenkr".to_owned()])
    );
    bus.shutdown();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_shopify_offer_details_reach_the_outbox_once_on_replay() {
    let db = Database::new();
    let server = Server::new();
    let storage = server.client();
    let app = initial_fill_saga_app(&db, &server);
    let saga_id = Uuid::now_v7();
    let retailer = RetailerCode::AnkerAu;
    DieselPgSagaRepository::new(db.pool.clone())
        .insert(ShopifyGraphQlInitialFillSaga::new(
            saga_id,
            Uuid::now_v7(),
            retailer,
        ))
        .unwrap();

    let product: Value = from_str(include_str!(
        "../../tests/fixtures/anker/ankerau_product.json"
    ))
    .unwrap();
    let context = json!({"country":"AU","language":"EN","currency":"AUD"});
    let artifact = json!({"schema_version":2, "context":context, "product":product, "pages":[]});
    let fetched_at = Utc::now();
    let fetched = OfferDetailsFetched {
        context: Some(from_value(context).unwrap()),
        retailer_code: retailer,
        source: SourceType::GraphQl {
            product_id: product["id"].as_str().unwrap().into(),
        },
        fetched_at,
        storage: storage.store(&artifact.to_string(), fetched_at).unwrap(),
    };
    let event_id = Uuid::now_v7();
    sql_query(
        "INSERT INTO event_entries (id,event_type,payload,meta,status) VALUES ($1,$2,$3,$4,0)",
    )
    .bind::<SqlUuid, _>(event_id)
    .bind::<Text, _>(OfferDetailsFetched::NAME)
    .bind::<Text, _>(to_string(&fetched).unwrap())
    .bind::<Jsonb, _>(json!({"event_id":event_id,"correlation_id":saga_id,"source":"test"}))
    .execute(&mut db.pool.get().unwrap())
    .unwrap();

    run(&server, &app, "/api/events/run");
    for _ in 0..2 {
        run(&server, &app, "/api/commands/run");
        run(&server, &app, "/api/events/run");
    }
    let events = rows(&db, "event_entries");
    let events = events.as_array().unwrap();
    assert_eq!(events.len(), 3);
    let terminal = events
        .iter()
        .find(|row| row["event_type"] == OfferDetailsStructuralized::NAME)
        .unwrap();
    let event: OfferDetailsStructuralized =
        from_str(terminal["payload"].as_str().unwrap()).unwrap();
    assert_eq!(event.retailer_code, fetched.retailer_code);
    assert_eq!(event.source, fetched.source);
    assert_eq!(event.fetched_at, fetched.fetched_at);
    assert_eq!(event.storage.external_id, fetched.storage.object_storage_id);
    assert_eq!(event.storage.external_id, event.storage.object_storage_id);
    assert_eq!(event.storage.scraped_at, Some(fetched_at));
    assert_eq!(event.storage.scraped_at, Some(fetched_at));
    let extracted = storage
        .get_offer("offer_details_extracted", &event.storage)
        .unwrap();
    let extracted: Value = from_str(&extracted).unwrap();
    assert_eq!(extracted["context"], artifact["context"]);
    let structuralized = storage
        .get_offer("offer_details_structuralized", &event.storage)
        .unwrap();
    let structuralized: Value = from_str(&structuralized).unwrap();
    let variant = &structuralized["variations"][0];

    let outbox = rows(&db, "outbox_entries");
    let [message] = outbox.as_array().unwrap().as_slice() else {
        panic!("expected one import, got {outbox}");
    };
    let variant_id = variant["identifiers"]["id"].as_u64().unwrap();
    assert_eq!(
        message["id"],
        json!(Uuid::new_v5(
            &event.storage.object_storage_id,
            &variant_id.to_be_bytes()
        ))
    );
    assert_eq!(message["meta"]["event_id"], terminal["id"]);
    assert_eq!(message["meta"]["correlation_id"], json!(saga_id));
    assert_eq!(message["meta"]["routing_key"], "offers");
    let payload: Value = from_str(message["payload"].as_str().unwrap()).unwrap();
    assert_eq!(payload["retailer_code"], "ankerau");
    assert_eq!(payload["scraped_at"], json!(fetched_at));
    assert_eq!(payload["url"], variant["content"]["url"]);
    assert_eq!(payload["price"], json!({"amount":10995,"currency":"AUD"}));

    sql_query("UPDATE event_entries SET status=0, scheduled_at=now()")
        .execute(&mut db.pool.get().unwrap())
        .unwrap();
    run(&server, &app, "/api/events/run");
    run(&server, &app, "/api/commands/run");
    assert_eq!(rows(&db, "command_entries").as_array().unwrap().len(), 2);
    assert_eq!(rows(&db, "event_entries").as_array().unwrap().len(), 3);
    assert_eq!(rows(&db, "outbox_entries"), outbox);
    assert_eq!(server.data.lock().unwrap().records.len(), 3);
}

fn initial_fill_saga_app(db: &Database, server: &Server) -> Router {
    let platform = Platform {
        db: db.pool.clone(),
        http: Client,
        offer_http: Default::default(),
        object_storage: server.client(),
    };
    let mut bus = Bus::default();
    register_sitemap_steps(&mut bus, &platform);
    register_html_steps(&mut bus, &platform);
    register_offer_processing_steps(&mut bus, &platform);
    register_initial_fill_listeners(&mut bus, &platform);
    register_shopify_steps(&mut bus, &platform);
    let guard = server.runtime.enter();
    let handle = bus.start(db.pool.clone()).unwrap();
    drop(guard);
    app(AppState {
        bus: Some(handle),
        ..Default::default()
    })
}

fn seed_initial_request(
    db: &Database,
    saga_id: Uuid,
    retailer: &str,
    strategy: &str,
    correlation: Option<Uuid>,
) -> Uuid {
    let event = json!({"saga_id":saga_id,"retailer_code":retailer,"strategy":strategy});
    seed_event(db, "InitialFillRequested", &event, correlation)
}

fn seed_event(db: &Database, name: &str, event: &Value, correlation: Option<Uuid>) -> Uuid {
    let event_id = Uuid::now_v7();
    sql_query("INSERT INTO event_entries (id,event_type,payload,meta,status) VALUES ($1,$2,$3,$4,0)")
        .bind::<SqlUuid,_>(event_id)
        .bind::<Text,_>(name)
        .bind::<Text,_>(event.to_string())
        .bind::<Jsonb,_>(json!({"event_id":event_id,"correlation_id":correlation,"causation_id":null,"source":"test"}))
        .execute(&mut db.pool.get().unwrap()).unwrap();
    event_id
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn shopify_saga_token_command_and_failed_requests() {
    let db = Database::new();
    let server = Server::new();
    let app = initial_fill_saga_app(&db, &server);
    let saga = Uuid::now_v7();
    let correlation = Uuid::now_v7();
    seed_initial_request(&db, saga, "ankerau", "Cafe24", None);
    run(&server, &app, "/api/events/run");
    assert_eq!(rows(&db, "command_entries"), json!([]));

    let event = seed_initial_request(&db, saga, "ankerau", "ShopifyGraphQl", Some(correlation));
    run(&server, &app, "/api/events/run");
    let commands = rows(&db, "command_entries");
    let token = &commands[0];
    assert_eq!(commands.as_array().unwrap().len(), 1);
    assert_eq!(token["command_type"], "AcquireStorefrontToken");
    assert_eq!(token["meta"]["correlation_id"], json!(saga));
    assert_eq!(token["meta"]["causation_id"], json!(event));
    assert_eq!(token["meta"]["source"], "event:InitialFillRequested");
    let payload: Value = from_str(token["payload"].as_str().unwrap()).unwrap();
    assert!(payload.get("run_id").is_none());
    assert_eq!(payload["retailer_code"], "ankerau");

    // Reusing the ID for a different retailer must not queue catalog commands.
    seed_initial_request(&db, saga, "ugreeneu", "ShopifyGraphQl", Some(correlation));
    // A disconnected retailer cannot produce a starting batch.
    seed_initial_request(
        &db,
        Uuid::now_v7(),
        "micom",
        "ShopifyGraphQl",
        Some(correlation),
    );
    seed_initial_request(&db, Uuid::now_v7(), "ankerau", "ShopifyGraphQl", None);
    let (_, body) = post(&server, &app, "/api/events/run", json!({"limit":10}));
    assert_eq!(body["errors"].as_array().unwrap().len(), 3, "{body}");
    assert_eq!(rows(&db, "command_entries"), commands);
    assert_eq!(
        json_query(
            &mut db.pool.get().unwrap(),
            "SELECT to_jsonb(count(*)) AS value FROM shopify_graphql_initial_fills"
        ),
        json!(1)
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn shopify_saga_queues_one_catalog_command_per_locale() {
    let db = Database::new();
    let server = Server::new();
    let app = initial_fill_saga_app(&db, &server);
    let saga = Uuid::now_v7();
    let event = seed_initial_request(
        &db,
        saga,
        "ugreeneu",
        "ShopifyGraphQl",
        Some(Uuid::now_v7()),
    );
    run(&server, &app, "/api/events/run");
    let rows = rows(&db, "command_entries");
    let commands = rows.as_array().unwrap();
    let mut expected = get_storefront_config(RetailerCode::UgreenEu)
        .unwrap()
        .initial_fill_contexts()
        .unwrap();
    assert_eq!(commands.len(), expected.len());
    for row in commands {
        assert_eq!(row["command_type"], "FetchProductCatalogIdentifiers");
        assert_eq!(row["meta"]["causation_id"], json!(event));
        assert_eq!(row["meta"]["correlation_id"], json!(saga));
        assert_eq!(row["meta"]["source"], "event:InitialFillRequested");
        let command: FetchProductCatalogIdentifiers =
            from_str(row["payload"].as_str().unwrap()).unwrap();
        assert_eq!(command.retailer_code, RetailerCode::UgreenEu);
        assert!(command.storefront_token.is_none());
        let context = command.context.unwrap();
        let index = expected
            .iter()
            .position(|c| *c == context)
            .expect("unexpected or duplicate locale");
        expected.swap_remove(index);
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn sitemap_saga_and_standalone_retrievals_queue_one_command_per_transition() {
    let db = Database::new();
    let server = Server::new();
    let app = initial_fill_saga_app(&db, &server);
    let saga = Uuid::now_v7();
    let request = seed_initial_request(
        &db,
        saga,
        "ankerau",
        "ShopifySitemapPlusHtml",
        Some(Uuid::now_v7()),
    );
    run(&server, &app, "/api/events/run");
    let initial = rows(&db, "command_entries");
    let [retrieve] = initial.as_array().unwrap().as_slice() else {
        panic!("expected a single retrieval");
    };
    assert_eq!(retrieve["command_type"], "RetrieveSitemap");
    assert_eq!(retrieve["meta"]["correlation_id"], json!(saga));
    assert_eq!(retrieve["meta"]["causation_id"], json!(request));

    sql_query("UPDATE event_entries SET status=0, scheduled_at=now() WHERE id=$1")
        .bind::<SqlUuid, _>(request)
        .execute(&mut db.pool.get().unwrap())
        .unwrap();
    run(&server, &app, "/api/events/run");
    assert_eq!(rows(&db, "command_entries"), initial);

    let payload: Value = from_str(retrieve["payload"].as_str().unwrap()).unwrap();
    let mut expected = vec![];
    for correlation in [saga, Uuid::now_v7()] {
        let mut fact = json!({
            "retrieval_id": if correlation == saga {payload["retrieval_id"].clone()} else {json!(Uuid::now_v7())},
            "processed_sitemap_id": Uuid::now_v7(),
            "consolidated_sitemap_id": Uuid::now_v7(),
            "retailer_code": "ankerau", "document_count": 1, "url_count": 1,
            "product_count": 1, "content_count": 0, "catalog_count": 0,
            "unknown_count": 0, "not_interested_count": 0, "offer_id": Uuid::now_v7(),
            "url": "https://shop.test/product", "scraped_at": null,
        });
        for (event, command) in [
            ("SitemapRetrieved", "ProcessSitemap"),
            ("SitemapProcessed", "ConsolidateSitemap"),
            ("SitemapConsolidated", "DiscoverOffers"),
            ("OfferDiscovered", "DownloadOfferPage"),
            ("OfferPageWasDownloaded", "ExtractOfferPage"),
            ("OfferPageWasExtracted", "NormalizeOffer"),
            ("OfferWasNormalized", "StructuralizeOffer"),
            ("OfferWasStructuralized", "ProcessOfferForExport"),
        ] {
            let cause = seed_event(&db, event, &fact, Some(correlation));
            expected.push((cause, correlation, event, command, fact.clone()));
        }
        fact["product_count"] = json!(0);
        seed_event(&db, "SitemapConsolidated", &fact, Some(correlation));
    }
    run(&server, &app, "/api/events/run");
    let commands = rows(&db, "command_entries");
    let commands = commands.as_array().unwrap();
    assert_eq!(commands.len(), expected.len() + 1);
    for (cause, correlation, event, command, fact) in expected {
        let row = commands
            .iter()
            .find(|row| row["meta"]["causation_id"] == json!(cause))
            .unwrap();
        if command != "ProcessOfferForExport" {
            assert_eq!(row["id"] == json!(cause), correlation == saga);
        }
        assert_eq!(row["command_type"], command);
        assert_eq!(row["meta"]["correlation_id"], json!(correlation));
        assert_eq!(row["meta"]["source"], format!("event:{event}"));
        let payload: Value = from_str(row["payload"].as_str().unwrap()).unwrap();
        for (key, value) in payload.as_object().unwrap() {
            if key != "schema_version" {
                assert_eq!(value, &fact[key], "{command}.{key}");
            }
        }
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn sitemap_saga_propagates_startup_and_repository_errors() {
    let db = Database::new();
    let server = Server::new();
    let app = initial_fill_saga_app(&db, &server);
    let saga = Uuid::now_v7();
    let correlation = Some(Uuid::now_v7());
    seed_initial_request(&db, saga, "ankerau", "ShopifySitemapPlusHtml", correlation);
    run(&server, &app, "/api/events/run");
    let initial = rows(&db, "command_entries");
    seed_initial_request(&db, saga, "ugreenca", "ShopifySitemapPlusHtml", correlation);
    seed_initial_request(
        &db,
        Uuid::now_v7(),
        "ankerau",
        "ShopifySitemapPlusHtml",
        None,
    );
    let (_, body) = post(&server, &app, "/api/events/run", json!({"limit": 10}));
    assert_eq!(body["errors"].as_array().unwrap().len(), 2, "{body}");
    assert_eq!(rows(&db, "command_entries"), initial);

    // Simulate an unavailable repository inside this test's isolated schema.
    sql_query("ALTER TABLE shopify_graphql_initial_fills RENAME TO unavailable_sagas")
        .execute(&mut db.pool.get().unwrap())
        .unwrap();
    seed_initial_request(
        &db,
        Uuid::now_v7(),
        "ankerau",
        "ShopifySitemapPlusHtml",
        correlation,
    );
    seed_event(
        &db,
        "SitemapRetrieved",
        &json!({"retrieval_id": Uuid::now_v7(), "retailer_code": "ankerau", "document_count": 1}),
        Some(saga),
    );
    let (_, body) = post(&server, &app, "/api/events/run", json!({"limit": 10}));
    assert!(body["errors"].as_array().unwrap().len() >= 2, "{body}");
    assert_eq!(rows(&db, "command_entries"), initial);
}
