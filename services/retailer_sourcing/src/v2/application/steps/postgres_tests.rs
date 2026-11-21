//! Durable end-to-end tests in an isolated schema, with local object-storage fixtures.
use super::tests::{connection, token_value};
use crate::app;
use crate::platform::{
    bus::{Bus, Event},
    state::AppState,
    test_db::{Database, json_query},
    test_objects::Server,
};
use crate::saga::shopify_graphql_initial_fill::register as register_initial_fill_saga;
use crate::steps::prelude::acquire_storefront_token as token;
use crate::steps::prelude::{offer_details as details, product_catalog_ids as catalog};
use crate::steps::prelude::{register_derived_offer_steps, register_initialization};
use crate::v2::incoming::InitialFillContexts;
use axum::Router;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use catalog::Handler as CatalogHandler;
use catalog::ProductCatalogIdsRetrieved;

use chrono::Utc;
use details::BUCKET;
use details::Handler as OfferDetailsHandler;
use details::OfferDetailsFetched;
use details::fetch_details_for_catalog_products;
use diesel::sql_query;
use diesel::{
    connection::SimpleConnection,
    prelude::*,
    sql_types::{Jsonb, Text, Uuid as SqlUuid},
};
use retailer_data_extractor::get_storefront_config;
use retailer_sauce::prelude::GraphqlRequest;
use retailer_sauce::prelude::Http;
use retailer_sauce::prelude::StorefrontToken;
use serde_json::from_slice;
use serde_json::from_str;
use serde_json::from_value;
use serde_json::to_string;
use serde_json::to_value;
use serde_json::{Value, json};
use shared::retailer::RetailerCode;
use token::Fetch as StorefrontTokenFetch;
use token::Handler as AcquireStorefrontTokenHandler;
use tower::ServiceExt;
use uuid::Uuid;

struct Documents;
impl StorefrontTokenFetch for Documents {
    fn configured_token(&self, _: &str) -> Result<Option<StorefrontToken>, String> {
        Ok(Some(token_value()))
    }
    fn get(&self, _: &str) -> Result<String, String> {
        Ok(r#"<script>storeTokens:{au:'synthetic-au-token',ca:'synthetic-ca-token',us:'public-us',uk:'public-uk',de:'public-de',fr:'public-fr','eu-en':'public-eu','eu-pl':'public-pl',nz:'public-nz',vn:'public-vn'}</script>"#.into())
    }
}
struct Source;
impl Http for Source {
    fn post(
        &self,
        _: &str,
        operation: &GraphqlRequest,
        _: Option<&StorefrontToken>,
    ) -> Result<Value, String> {
        let context = json!({"country": operation.variables.get("country").cloned().unwrap_or(json!("DE")), "language": operation.variables["language"]});
        match operation.operation_name.as_str() {
            "CatalogIdentifiers" => Ok(
                json!({"data":{"products":connection((1..=3).map(|i|json!({"cursor":format!("c{i}"),"node":{"id":format!("gid://shopify/Product/{i}")}})).collect(),false,Some("c3"))},"extensions":{"context":context}}),
            ),
            "OfferDetails" => {
                let id = operation.variables["id"].as_str().unwrap();
                let mut value: Value = from_str(include_str!(
                    "../../../../tests/fixtures/anker/ankerau_product.json"
                ))
                .unwrap();
                value["id"] = json!(id);
                Ok(json!({"data":{"product":value},"extensions":{"context":context}}))
            }
            _ => Err("unexpected continuation".into()),
        }
    }
}
fn rows(db: &Database, table: &str) -> Value {
    json_query(
        &mut db.pool.get().unwrap(),
        &format!("SELECT coalesce(jsonb_agg(to_jsonb(t)), '[]') AS value FROM {table} t"),
    )
}
fn post(server: &Server, app: &Router, path: &str, body: Value) -> (StatusCode, Value) {
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
        let value = if status == StatusCode::NO_CONTENT {
            assert!(body.is_empty());
            Value::Null
        } else {
            from_slice(&body).unwrap()
        };
        (status, value)
    })
}
fn run(server: &Server, app: &Router, path: &str) {
    let (status, body) = post(server, app, path, json!({"limit":100}));
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["errors"], json!([]), "{body}");
}
#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_initial_fill_extracts_stored_details() {
    run_initial_fill(RetailerCode::AnkerAu);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_canadian_initial_fill_extracts_stored_details() {
    run_initial_fill(RetailerCode::AnkerCa);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_remaining_inventory_locales_extract_stored_details() {
    for retailer in RetailerCode::ALL {
        if !matches!(retailer, RetailerCode::AnkerAu | RetailerCode::AnkerCa)
            && get_storefront_config(retailer).is_some()
        {
            run_initial_fill(retailer);
        }
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_bulk_initial_fill_validates_before_dispatch_and_preserves_run_identities() {
    let db = Database::new();
    let server = Server::new();
    let mut bus = Bus::default();
    bus.handle(AcquireStorefrontTokenHandler(Documents));
    bus.handle(CatalogHandler(Source));
    let guard = server.runtime.enter();
    let bus = bus.start(db.pool.clone()).unwrap();
    drop(guard);
    let app = app(AppState {
        bus: Some(bus.clone()),
        db: Some(db.pool.clone()),
        ..Default::default()
    });
    let (status, _) = post(
        &server,
        &app,
        "/api/initial-offer-fills",
        json!({"retailers":["ankerau", "unknown"]}),
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(rows(&db, "command_entries").as_array().unwrap().is_empty());

    let (status, _) = post(
        &server,
        &app,
        "/api/initial-offer-fills",
        json!({"retailers":["ankerau","ankerkr"]}),
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(rows(&db, "command_entries").as_array().unwrap().is_empty());
    let retailers = ["ankerau", "ankerca", "ankeritalycom"];
    let (status, receipts) = post(
        &server,
        &app,
        "/api/initial-offer-fills",
        json!({"retailers": retailers}),
    );
    assert_eq!(status, StatusCode::ACCEPTED);
    let commands = json_query(
        &mut db.pool.get().unwrap(),
        "SELECT jsonb_agg(to_jsonb(c) ORDER BY id) AS value FROM command_entries c",
    );
    let commands = commands.as_array().unwrap();
    assert_eq!(commands.len(), retailers.len());
    assert_eq!(receipts["fills"].as_array().unwrap().len(), retailers.len());
    for (command, receipt) in commands.iter().zip(receipts["fills"].as_array().unwrap()) {
        assert_eq!(command["id"], receipt["correlation_id"]);
    }
    for (command, retailer) in commands.iter().zip(retailers) {
        assert_eq!(command["command_type"], "InitializeRetailerFill");
        let payload: Value = from_str(command["payload"].as_str().unwrap()).unwrap();
        assert_eq!(payload, json!({"retailer_code": retailer}));
        assert_eq!(command["meta"]["command_id"], command["id"]);
        assert_eq!(command["meta"]["correlation_id"], command["id"]);
    }

    db.pool.get().unwrap().batch_execute(
        "ALTER TABLE command_entries ADD CHECK (payload::jsonb->>'retailer_code' <> 'ankerca') NOT VALID"
    ).unwrap();
    let (status, failure) = post(
        &server,
        &app,
        "/api/initial-offer-fills",
        json!({"retailers": ["ankerau", "ankerca", "ankeritalycom"]}),
    );
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let message = failure["error"].as_str().unwrap();
    assert!(!message.is_empty());
    assert_eq!(failure, json!({"error": message}));
    assert_eq!(rows(&db, "command_entries").as_array().unwrap().len(), 4);
    bus.shutdown();
}

fn run_initial_fill(retailer: RetailerCode) {
    let contexts = get_storefront_config(retailer)
        .unwrap()
        .initial_fill_contexts()
        .unwrap();
    let attempts_token = get_storefront_config(retailer).unwrap().attempts_token();
    let expected_commands = 1 + usize::from(attempts_token) + contexts.len() * 10;
    let db = Database::new();
    let server = Server::new();
    let mut bus = Bus::default();
    register_initialization(&mut bus);
    register_initial_fill_saga(&mut bus, &db.pool);
    bus.handle(AcquireStorefrontTokenHandler(Documents));
    bus.handle(CatalogHandler(Source));
    bus.handle(OfferDetailsHandler {
        fetcher: Source,
        store: server.client(),
    });
    register_derived_offer_steps(&mut bus, &db.pool, &server.client());

    let guard = server.runtime.enter();
    let bus = bus.start(db.pool.clone()).unwrap();
    drop(guard);
    let app = app(AppState {
        bus: Some(bus.clone()),
        db: Some(db.pool.clone()),
        ..Default::default()
    });
    let (status, _) = post(
        &server,
        &app,
        "/api/initial-offer-fills",
        json!({"retailers":[retailer.as_str()]}),
    );
    assert_eq!(status, StatusCode::ACCEPTED);
    run(&server, &app, "/api/commands/run");
    run(&server, &app, "/api/events/run");
    let correlation_id = json_query(
        &mut db.pool.get().unwrap(),
        "SELECT meta->'correlation_id' AS value FROM command_entries WHERE command_type = 'InitializeRetailerFill'",
    );
    for _ in 0..5 {
        run(&server, &app, "/api/commands/run");
        run(&server, &app, "/api/events/run");
    }
    let saga = json_query(
        &mut db.pool.get().unwrap(),
        "SELECT to_jsonb(s) AS value FROM sagas s",
    );
    assert_eq!(saga["status"], "succeeded");
    assert_ne!(saga["id"], saga["correlation_id"]);
    assert_eq!(
        saga["payload"]["catalogs"].as_array().unwrap().len(),
        contexts.len()
    );
    for catalog in saga["payload"]["catalogs"].as_array().unwrap() {
        assert_eq!(catalog["products"].as_object().unwrap().len(), 3);
        assert!(
            catalog["products"]
                .as_object()
                .unwrap()
                .values()
                .all(|product| product["structuralized"]["object"].is_string())
        );
    }
    let commands = rows(&db, "command_entries");
    assert_eq!(commands.as_array().unwrap().len(), expected_commands);
    assert_eq!(
        commands
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["command_type"] == "AcquireStorefrontToken")
            .count(),
        usize::from(attempts_token)
    );
    if !attempts_token {
        for row in commands.as_array().unwrap() {
            let payload: Value = from_str(row["payload"].as_str().unwrap()).unwrap();
            assert!(payload["storefront_token"].is_null());
        }
    }
    assert!(
        commands
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["status"] == 5)
    );
    assert!(
        commands
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["meta"]["correlation_id"] == correlation_id)
    );
    let events = rows(&db, "event_entries");
    assert!(
        events
            .as_array()
            .unwrap()
            .iter()
            .all(|event| { event["meta"]["correlation_id"] == correlation_id })
    );
    let terminal: Vec<_> = events
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_type"] == "OfferDetailsFetched")
        .collect();
    assert_eq!(terminal.len(), contexts.len() * 3);
    // Check the actual upload destination against object storage's existing
    // contract, independently of the handler's bucket constant.
    let extracted: Vec<_> = events
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["event_type"] == "OfferDetailsExtracted")
        .map(|row| from_str::<Value>(row["payload"].as_str().unwrap()).unwrap())
        .collect();
    assert_eq!(extracted.len(), terminal.len());
    for row in terminal {
        assert_eq!(row["status"], 5);
        let event: OfferDetailsFetched = from_str(row["payload"].as_str().unwrap()).unwrap();
        assert_eq!(event.retailer_code, retailer);
        let output = extracted
            .iter()
            .find(|output| output["fetched"] == to_value(&event).unwrap())
            .unwrap();
        let storage = from_value(output["storage"].clone()).unwrap();
        let document = server
            .client()
            .get_offer("offer_details_extracted", &storage)
            .unwrap();
        let document: Value = from_str(&document).unwrap();
        assert_eq!(document["context"], to_value(&event.context).unwrap());
        assert!(!document["product"]["identifiers"]["id"].is_null());
        let artifact = server.client().get_offer(BUCKET, &event.storage).unwrap();
        let artifact: Value = from_str(&artifact).unwrap();
        assert_eq!(artifact["schema_version"], 2);
        let requested = &artifact["pages"][0]["request"]["variables"];
        assert!(
            contexts
                .iter()
                .any(|c| to_value(&c.country).unwrap() == requested["country"]
                    && c.language == requested["language"])
        );
        assert_eq!(artifact["context"], to_value(&event.context).unwrap());
        assert!(
            artifact["pages"][0]["request"]["query"]
                .as_str()
                .unwrap()
                .contains("language: $language")
        );
        assert_eq!(
            artifact["product"]["priceRange"]["minVariantPrice"]["currencyCode"],
            "AUD"
        );
        assert_eq!(artifact["pages"].as_array().unwrap().len(), 1);
    }
    run(&server, &app, "/api/events/run");
    assert_eq!(
        rows(&db, "command_entries").as_array().unwrap().len(),
        expected_commands
    );
    let outbox = rows(&db, "outbox_entries");
    let messages = outbox.as_array().unwrap();
    assert_eq!(messages.len(), contexts.len() * 3);
    for message in messages {
        let meta = &message["meta"];
        assert_eq!(meta["routing_key"], "offers");
        assert_eq!(meta["correlation_id"], correlation_id);
        assert_eq!(meta["event_type"], "OfferDetailsStructuralized");
        let row = events
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == meta["event_id"])
            .unwrap();
        let event: Value = from_str(row["payload"].as_str().unwrap()).unwrap();
        let storage = from_value(event["storage"].clone()).unwrap();
        let body = server
            .client()
            .get_offer("offer_details_structuralized", &storage)
            .unwrap();
        let document: Value = from_str(&body).unwrap();
        let variation = &document["variations"][0];
        let payload: Value = from_str(message["payload"].as_str().unwrap()).unwrap();
        assert_eq!(payload["retailer_code"], retailer.as_str());
        assert_eq!(payload["url"], variation["content"]["url"]);
        assert_eq!(payload["title"], variation["content"]["title"]);
        assert_eq!(payload["price"], variation["pricing"]["Standard"]);
        assert_eq!(
            payload["localization"]["target_market"]["code"],
            variation["market"]
        );
        assert_eq!(
            payload["scraped_at"],
            event["extracted"]["fetched"]["fetched_at"]
        );
    }
    db.pool.get().unwrap().batch_execute("UPDATE event_entries SET status=0, scheduled_at=now() WHERE event_type IN ('OfferDetailsExtracted','OfferDetailsStructuralized')").unwrap();
    run(&server, &app, "/api/events/run");
    assert_eq!(
        rows(&db, "command_entries").as_array().unwrap().len(),
        expected_commands
    );
    assert_eq!(
        rows(&db, "outbox_entries").as_array().unwrap().len(),
        messages.len()
    );
    if retailer == RetailerCode::AnkerAu {
        assert_inbox_delivery(&db, &server, messages);
    }
    bus.shutdown();
}
fn assert_inbox_delivery(sender: &Database, server: &Server, messages: &[Value]) {
    use crate::platform::bus_api::outbox_postgres_tests::{inbox, state};
    use crate::platform::bus_api::outbox_tests::run as run_outbox;

    let receiver = Database::new();
    let inbox = server.runtime.block_on(inbox(&receiver));
    let state = {
        let _guard = server.runtime.enter();
        state(sender, &inbox.url)
    };
    let (_, result) = server
        .runtime
        .block_on(run_outbox(state, json!({"limit":100})));
    assert_eq!(result, json!({"processed": messages.len(), "errors":[]}));
    let inbox = rows(&receiver, "inbox_entries");
    let received = inbox.as_array().unwrap();
    assert_eq!(received.len(), messages.len());
    for message in messages {
        let row = received.iter().find(|r| r["id"] == message["id"]).unwrap();
        assert_eq!(row["meta"]["routing_key"], "ImportOfferDetails");
        assert_eq!(
            row["meta"]["correlation_id"],
            message["meta"]["correlation_id"]
        );
        let payload: Value = from_str(row["payload"].as_str().unwrap()).unwrap();
        assert_eq!(
            payload,
            from_str::<Value>(message["payload"].as_str().unwrap()).unwrap()
        );
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_partial_fanout_replay_deduplicates_command_rows() {
    let db = Database::new();
    let server = Server::new();
    let event = ProductCatalogIdsRetrieved {
        context: None,

        retailer_code: RetailerCode::AnkerAu,
        storefront_token: Some(token_value()),
        product_count: 3,
        loop_count: 1,
        product_ids: (1..=3)
            .map(|i| format!("gid://shopify/Product/{i}"))
            .collect(),
        retrieved_at: Utc::now(),
    };
    let failed_id = fetch_details_for_catalog_products(Uuid::nil(), &event)[1].id;
    let mut conn = db.pool.get().unwrap();
    conn.batch_execute(&format!("CREATE FUNCTION fail_fanout() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.id = '{failed_id}'::uuid THEN RAISE EXCEPTION 'fixture failure'; END IF; RETURN NEW; END $$; CREATE TRIGGER fail_fanout BEFORE INSERT ON command_entries FOR EACH ROW EXECUTE FUNCTION fail_fanout();")).unwrap();
    let event_id = Uuid::now_v7();
    sql_query("INSERT INTO event_entries (id,event_type,payload,meta,status) VALUES ($1,$2,$3,$4,0)")
        .bind::<SqlUuid,_>(event_id).bind::<Text,_>(ProductCatalogIdsRetrieved::NAME)
        .bind::<Text,_>(to_string(&event).unwrap())
        .bind::<Jsonb,_>(json!({"event_id":event_id,"correlation_id":Uuid::now_v7(),"causation_id":null,"source":"test"}))
        .execute(&mut conn).unwrap();
    drop(conn);
    let mut bus = Bus::default();
    bus.on_many_keyed(|event| fetch_details_for_catalog_products(Uuid::nil(), event));
    let guard = server.runtime.enter();
    let bus = bus.start(db.pool.clone()).unwrap();
    drop(guard);
    let app = app(AppState {
        bus: Some(bus.clone()),
        db: Some(db.pool.clone()),
        ..Default::default()
    });
    let (_, body) = post(&server, &app, "/api/events/run", json!({"limit":10}));
    assert_eq!(body["errors"].as_array().unwrap().len(), 1);
    assert_eq!(rows(&db, "command_entries").as_array().unwrap().len(), 1);
    db.pool.get().unwrap().batch_execute("DROP TRIGGER fail_fanout ON command_entries; UPDATE event_entries SET scheduled_at=now() WHERE status=4").unwrap();
    run(&server, &app, "/api/events/run");
    assert_eq!(rows(&db, "command_entries").as_array().unwrap().len(), 3);
    // Explicit replay of an already-processed batch must not insert more commands.
    db.pool
        .get()
        .unwrap()
        .batch_execute("UPDATE event_entries SET status=0, scheduled_at=now()")
        .unwrap();
    run(&server, &app, "/api/events/run");
    assert_eq!(rows(&db, "command_entries").as_array().unwrap().len(), 3);
    bus.shutdown();
}
