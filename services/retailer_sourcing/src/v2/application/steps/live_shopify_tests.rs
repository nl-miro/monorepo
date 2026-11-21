//! Explicit live smoke test: acquire a public token, read the whole catalog, store one complete product.
use crate::platform::{bus::Handle, http, object_storage::ObjectStorage};
use crate::steps::prelude::acquire_storefront_token as token;
use crate::steps::prelude::{offer_details as details, product_catalog_ids as catalog};
use catalog::FetchProductCatalogIdentifiers;
use catalog::Handler as CatalogHandler;
use catalog::start_catalog_for_locales;
use details::Handler as OfferDetailsHandler;
use details::fetch_details_for_catalog_products;
use details::{BUCKET, SourceType};
use http::Client;
use http::OfferClient;
use retailer_data_extractor::get_storefront_config;
use serde_json::Value;
use serde_json::from_str;
use shared::retailer::RetailerCode;
use std::collections::HashSet;
use std::env::var;
use token::AcquireStorefrontToken;
use token::Handler as AcquireStorefrontTokenHandler;
use tokio::runtime::Runtime;
use uuid::Uuid;

#[test]
#[ignore = "requires live network and OBJECT_STORAGE_URL/TOKEN; run make test-initial-fill-live"]
fn anker_au_token_catalog_and_one_complete_offer() {
    run(RetailerCode::AnkerAu);
}

#[test]
#[ignore = "requires live network and OBJECT_STORAGE_URL/TOKEN; run make test-initial-fill-live"]
fn anker_ca_english_token_catalog_and_one_complete_offer() {
    run(RetailerCode::AnkerCa);
}

#[test]
#[ignore = "requires live network and OBJECT_STORAGE_URL/TOKEN; run make test-initial-fill-live"]
fn remaining_anker_markets_catalog_and_one_complete_offer() {
    for retailer in RetailerCode::ALL {
        if !matches!(retailer, RetailerCode::AnkerAu | RetailerCode::AnkerCa)
            && get_storefront_config(retailer).is_some()
        {
            run(retailer);
        }
    }
}

fn run(retailer: RetailerCode) {
    assert_eq!(
        var("INITIAL_FILL_LIVE").as_deref(),
        Ok("1"),
        "set INITIAL_FILL_LIVE=1 explicitly"
    );
    let runtime = Runtime::new().unwrap();
    let objects = ObjectStorage::from_env(runtime.handle().clone()).unwrap();
    let config = get_storefront_config(retailer).unwrap();
    let command = if config.requires_token() {
        let event = AcquireStorefrontTokenHandler(Client)
            .handle(AcquireStorefrontToken {
                retailer_code: retailer,
                url: config.storefront_url().into(),
            })
            .unwrap()
            .remove(0);
        start_catalog_for_locales(Uuid::nil(), &event)
            .remove(0)
            .command
    } else {
        FetchProductCatalogIdentifiers {
            context: None,

            retailer_code: retailer,
            storefront_token: None,
        }
    };
    let http = OfferClient::configured().unwrap();
    let handler = CatalogHandler(http.clone());
    let mut events = handler.handle(command).unwrap();
    assert_eq!(events.len(), 1);
    let batch = events.remove(0);
    assert!(!batch.product_ids.is_empty());
    let unique: HashSet<_> = batch.product_ids.iter().collect();
    assert_eq!(unique.len(), batch.product_ids.len());
    let command = fetch_details_for_catalog_products(Uuid::nil(), &batch)
        .remove(0)
        .command;
    let event = OfferDetailsHandler {
        fetcher: http,
        store: objects.clone(),
    }
    .handle(command)
    .unwrap()
    .remove(0);
    let artifact = objects.get_offer(BUCKET, &event.storage).unwrap();
    let artifact: Value = from_str(&artifact).unwrap();
    assert_eq!(artifact["schema_version"], 1);
    assert!(!artifact["pages"].as_array().unwrap().is_empty());
}

#[test]
#[ignore = "requires PostgreSQL 18, live network and object storage; run make test-saga-live"]
fn live_api_anker_au_saga() {
    run_api(RetailerCode::AnkerAu);
}

#[test]
#[ignore = "requires PostgreSQL 18, live network and object storage; run make test-saga-live"]
fn live_api_ugreen_eu_saga() {
    run_api(RetailerCode::UgreenEu);
}

fn run_api(retailer: RetailerCode) {
    use crate::app;
    use crate::platform::Platform;
    use crate::platform::auth::BEARER_TOKEN;
    use crate::platform::bus::Bus;
    use crate::platform::object_storage::STRUCTURALIZED_OFFERS_BUCKET;
    use crate::platform::state::AppState;
    use crate::platform::test_db::{Database, json_query};
    use crate::steps::graphql::structuralize_offer_details::OfferDetailsStructuralized;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use serde_json::{from_slice, json, to_vec};
    use std::env::temp_dir;
    use std::fs::write;
    use std::thread::{scope, sleep};
    use std::time::{Duration, Instant};
    use tower::ServiceExt;

    assert_eq!(var("INITIAL_FILL_LIVE").as_deref(), Ok("1"));
    let runtime = Runtime::new().unwrap();
    let db = Database::new();
    let storage = ObjectStorage::from_env(runtime.handle().clone()).unwrap();
    let platform = Platform {
        db: db.pool.clone(),
        http: Client,
        // Live saga verification uses direct public storefront HTTP; proxy routing
        // is covered separately by the HTTP adapter tests.
        offer_http: OfferClient::default(),
        object_storage: storage.clone(),
    };
    use crate::platform::recovery::Recovery;
    use crate::register_graphql_pipeline;
    let mut bus = Bus::default();
    register_graphql_pipeline(&mut bus, &platform);
    let guard = runtime.enter();
    let mut bus = bus.start(db.pool.clone()).unwrap();
    bus.recovery = Some(Recovery::new(
        db.pool.clone(),
        var("DATABASE_URL_TEST").unwrap(),
    ));
    drop(guard);
    let app = app(AppState {
        db: Some(db.pool.clone()),
        bus: Some(bus.clone()),
        object_storage: Some(storage.clone()),
        ..Default::default()
    });
    let post = |path: &str, body: Value| {
        let response = runtime
            .block_on(
                app.clone().oneshot(
                    Request::post(path)
                        .header("content-type", "application/json")
                        .header("authorization", format!("Bearer {BEARER_TOKEN}"))
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                ),
            )
            .unwrap();
        let status = response.status();
        let bytes = runtime
            .block_on(to_bytes(response.into_body(), 16 * 1024 * 1024))
            .unwrap();
        assert!(
            status.is_success(),
            "{path}: {status} {}",
            String::from_utf8_lossy(&bytes)
        );
        if status == StatusCode::NO_CONTENT {
            Value::Null
        } else {
            from_slice(&bytes).unwrap()
        }
    };
    post(
        "/api/initial-offer-fills",
        json!({"retailers":[retailer.as_str()]}),
    );
    let start = Instant::now();
    loop {
        // Concurrent manual batches exercise reservation and saga locking while
        // keeping production background workers disabled.
        let commands: u64 = scope(|scope| {
            let batches: Vec<_> = (0..16)
                .map(|_| scope.spawn(|| post("/api/commands/run", json!({"limit":1}))))
                .collect();
            batches
                .into_iter()
                .map(|batch| batch.join().unwrap()["processed"].as_u64().unwrap())
                .sum()
        });
        let events = post("/api/events/run", json!({"limit":100}));
        let summary = json_query(
            &mut db.pool.get().unwrap(),
            "SELECT jsonb_build_object('status',status,'catalogs',jsonb_array_length(payload->'catalogs'),'products',(SELECT count(*) FROM jsonb_array_elements(payload->'catalogs') c, jsonb_each(c->'products') p)) AS value FROM sagas",
        );
        eprintln!(
            "{retailer:?} {:?}: {summary}; commands {}; events {}",
            start.elapsed(),
            commands,
            events["processed"]
        );
        if summary["status"] != "in_progress" {
            assert_eq!(
                summary["status"], "succeeded",
                "live fill ended unsuccessfully: {summary}"
            );
            assert!(summary["products"].as_u64().unwrap() > 0);
            break;
        }
        assert!(
            start.elapsed() < Duration::from_secs(3600),
            "live saga exceeded one hour: {summary}"
        );
        if commands == 0 && events["processed"] == 0 {
            sleep(Duration::from_secs(1));
        }
    }
    let events = json_query(
        &mut db.pool.get().unwrap(),
        "SELECT COALESCE(jsonb_agg(payload::jsonb), '[]') AS value FROM event_entries WHERE event_type='OfferDetailsStructuralized'",
    );
    // Preserve non-secret result references for diagnosis if artifact validation
    // fails after the isolated queue schema is cleaned up.
    let report = temp_dir().join(format!(
        "saga-live-{}-{}.json",
        retailer.as_str(),
        Uuid::now_v7()
    ));
    write(&report, to_vec(&events).unwrap()).unwrap();
    eprintln!(
        "{}: structured result references saved to {}",
        retailer.as_str(),
        report.display()
    );
    let mut contexts = HashSet::new();
    for event in events.as_array().unwrap() {
        let event: OfferDetailsStructuralized = from_str(&event.to_string()).unwrap();
        let context = event.extracted.fetched.context.as_ref().unwrap();
        contexts.insert((context.country.clone(), context.language.clone()));
        let document = storage
            .get_offer(STRUCTURALIZED_OFFERS_BUCKET, &event.storage)
            .unwrap();
        let document: Value = from_str(&document).unwrap();
        let raw = storage
            .get_offer(BUCKET, &event.extracted.fetched.storage)
            .unwrap();
        let raw: Value = from_str(&raw).unwrap();
        let SourceType::GraphQl { product_id } = &event.extracted.fetched.source else {
            panic!("expected GraphQL product");
        };
        assert_eq!(raw["product"]["id"], *product_id);
        let extracted = storage
            .get_offer("offer_details_extracted", &event.extracted.storage)
            .unwrap();
        let extracted: Value = from_str(&extracted).unwrap();
        let variants = extracted["product"]["variants"].as_array().unwrap();
        let variations = document["variations"].as_array().unwrap();
        assert!(!variations.is_empty());
        assert_eq!(variations.len(), variants.len());
        for (variation, variant) in variations.iter().zip(variants) {
            assert_eq!(variation["market"], context.country);
            assert_eq!(variation["content"]["language"], context.language);
            assert_eq!(variation["pricing"], variant["pricing"]);
            let pricing = &variation["pricing"];
            let price = pricing.get("Standard").unwrap_or(&pricing["Discounted"][0]);
            assert!(
                price.is_object(),
                "expected standard or discounted pricing: {pricing}"
            );
            assert_eq!(price["currency"], context.currency);
        }
    }
    eprintln!(
        "{retailer:?}: verified {} structured artifacts across {} resolved contexts in {:?}",
        events.as_array().unwrap().len(),
        contexts.len(),
        start.elapsed()
    );
    bus.shutdown();
}
