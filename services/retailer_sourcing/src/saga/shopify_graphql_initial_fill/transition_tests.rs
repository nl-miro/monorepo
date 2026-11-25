use super::listener::Listener;
use super::model::ResultRef;
use super::{ShopifyGraphQlInitialFillSaga, reconcile};
use crate::platform::bus::{Event, EventEnvelope, IntoEventCommand, KeyedCommand};
use crate::platform::object_storage::OfferStorageMetadata;
use crate::platform::test_db::{Database, json_query};
use crate::saga::saga_repository::SagaRepository;
use crate::steps::graphql::extract_offer_details::OfferDetailsExtracted;
use crate::steps::graphql::offer_details::{OfferDetailsFetched, SourceType};
use crate::steps::graphql::product_catalog_ids::ProductCatalogIdsRetrieved;
use crate::steps::graphql::structuralize_offer_details::OfferDetailsStructuralized;
use crate::steps::prelude::RetailerFillInitialized;
use chrono::Utc;
use diesel::sql_types::{Integer, Jsonb, Text, Uuid as SqlUuid};
use diesel::{RunQueryDsl, sql_query};
use kernel::NewEventMetadata;
use kernel::prelude::NewCommand;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::MarketContext;
use serde_json::{Value, from_slice, from_str, json, to_string};
use shared::retailer::RetailerCode;
use std::thread::spawn;
use std::time::Instant;
use uuid::Uuid;

const RETAILER: RetailerCode = RetailerCode::UgreenEu;
struct Fixture {
    db: Database,
    listener: Listener,
    correlation: Uuid,
    initialization: Uuid,
    catalog: KeyedCommand<NewCommand>,
    context: RequestedContext,
}
impl Fixture {
    fn new() -> Self {
        let db = Database::new();
        let repository = SagaRepository::new(db.pool.clone());
        let listener = Listener::new(repository.clone());
        let correlation = Uuid::now_v7();
        let initial = publish(
            &db,
            correlation,
            Uuid::now_v7(),
            RetailerFillInitialized {
                retailer_code: RETAILER,
            },
        );
        listener.on_retailer_fill_initialized(&initial).unwrap();
        repository
            .transition::<ShopifyGraphQlInitialFillSaga, _>(
                correlation,
                RETAILER,
                None,
                |record, _| {
                    record.payload.catalogs.truncate(1);
                    Ok(())
                },
            )
            .unwrap();
        let catalog = listener
            .on_retailer_fill_initialized(&initial)
            .unwrap()
            .remove(0)
            .encode()
            .unwrap();
        let payload: Value = from_str(&catalog.command.payload).unwrap();
        let context = from_str(&payload["context"].to_string()).unwrap();
        Self {
            db,
            listener,
            correlation,
            initialization: initial.metadata.unwrap().event_id,
            catalog,
            context,
        }
    }
    fn catalog(&self, ids: &[&str]) -> EventEnvelope<ProductCatalogIdsRetrieved> {
        publish(
            &self.db,
            self.correlation,
            self.catalog.id,
            ProductCatalogIdsRetrieved {
                retailer_code: RETAILER,
                context: Some(self.context.clone()),
                storefront_token: None,
                product_ids: ids.iter().map(|id| (*id).into()).collect(),
                product_count: ids.len(),
                loop_count: 1,
                retrieved_at: Utc::now(),
            },
        )
    }
    fn fetched(&self, command: Uuid, product: &str) -> EventEnvelope<OfferDetailsFetched> {
        publish(
            &self.db,
            self.correlation,
            command,
            OfferDetailsFetched {
                retailer_code: RETAILER,
                context: Some(MarketContext {
                    country: self.context.country.clone().unwrap_or("DE".into()),
                    language: self.context.language.clone(),
                    currency: "EUR".into(),
                }),
                source: SourceType::GraphQl {
                    product_id: product.into(),
                },
                fetched_at: Utc::now(),
                storage: storage(),
            },
        )
    }
    fn row(&self) -> Value {
        json_query(
            &mut self.db.pool.get().unwrap(),
            "SELECT to_jsonb(s) AS value FROM sagas s",
        )
    }
    fn command(
        &self,
        command: &KeyedCommand<NewCommand>,
        parent: Uuid,
        status: i32,
        attempts: i32,
    ) {
        sql_query("INSERT INTO command_entries (id,command_type,payload,meta,status,attempts) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT(id) DO UPDATE SET status=$5,attempts=$6")
            .bind::<SqlUuid,_>(command.id).bind::<Text,_>(&command.command.command_type).bind::<Text,_>(&command.command.payload)
            .bind::<Jsonb,_>(json!({"command_id":command.id,"correlation_id":self.correlation,"causation_id":parent}))
            .bind::<Integer,_>(status).bind::<Integer,_>(attempts).execute(&mut self.db.pool.get().unwrap()).unwrap();
    }
    fn event_status(&self, event: Uuid, status: i32, attempts: i32) {
        sql_query("UPDATE event_entries SET status=$2,attempts=$3 WHERE id=$1")
            .bind::<SqlUuid, _>(event)
            .bind::<Integer, _>(status)
            .bind::<Integer, _>(attempts)
            .execute(&mut self.db.pool.get().unwrap())
            .unwrap();
    }
}
fn storage() -> OfferStorageMetadata {
    OfferStorageMetadata {
        scraped_at: None,
        content_size: 100,
        object_storage_id: Uuid::now_v7(),
        external_id: Uuid::now_v7(),
    }
}
fn publish<E: Event>(db: &Database, correlation: Uuid, cause: Uuid, event: E) -> EventEnvelope<E> {
    let metadata = NewEventMetadata {
        event_id: Uuid::now_v7(),
        correlation_id: Some(correlation),
        causation_id: Some(cause),
        source: Some("test".into()),
        sequence: None,
    };
    sql_query(
        "INSERT INTO event_entries (id,event_type,payload,meta,status) VALUES ($1,$2,$3,$4,5)",
    )
    .bind::<SqlUuid, _>(metadata.event_id)
    .bind::<Text, _>(E::NAME)
    .bind::<Text, _>(to_string(&event).unwrap())
    .bind::<Jsonb, _>(json!(metadata))
    .execute(&mut db.pool.get().unwrap())
    .unwrap();
    EventEnvelope {
        event,
        metadata: Some(metadata),
    }
}
fn same(first: KeyedCommand<NewCommand>, second: KeyedCommand<NewCommand>) {
    assert_eq!(first.id, second.id);
    assert_eq!(first.command.payload, second.command.payload);
    let payload: Value = from_str(&first.command.payload).unwrap();
    assert!(payload.get("run_id").is_none());
    assert!(payload.get("saga_id").is_none());
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn canonical_results_replay_after_progress_and_concurrent_completion() {
    let f = Fixture::new();
    let catalog = f.catalog(&["one", "one", "two"]);
    let commands = f.listener.on_catalog(&catalog).unwrap();
    assert_eq!(commands.len(), 2);
    let mut finals = vec![];
    for (command, product) in commands.into_iter().zip(["one", "two"]) {
        let command = command.encode().unwrap();
        let fetched = f.fetched(command.id, product);
        let extraction = f
            .listener
            .on_fetched(&fetched)
            .unwrap()
            .remove(0)
            .encode()
            .unwrap();
        let duplicate = f.fetched(command.id, product);
        let replay = f
            .listener
            .on_fetched(&duplicate)
            .unwrap()
            .remove(0)
            .encode()
            .unwrap();
        assert_eq!(
            from_str::<Value>(&replay.command.payload).unwrap()["storage"]["object_storage_id"],
            json!(fetched.event.storage.object_storage_id)
        );
        let extracted = publish(
            &f.db,
            f.correlation,
            extraction.id,
            OfferDetailsExtracted {
                fetched: fetched.event.clone(),
                storage: storage(),
            },
        );
        let structuralize = f
            .listener
            .on_extracted(&extracted)
            .unwrap()
            .remove(0)
            .encode()
            .unwrap();
        let duplicate_extracted = publish(
            &f.db,
            f.correlation,
            extraction.id,
            OfferDetailsExtracted {
                fetched: extracted.event.fetched.clone(),
                storage: storage(),
            },
        );
        let replay_structuralize = f
            .listener
            .on_extracted(&duplicate_extracted)
            .unwrap()
            .remove(0)
            .encode()
            .unwrap();
        assert_eq!(structuralize.id, replay_structuralize.id);
        assert_eq!(
            structuralize.command.payload,
            replay_structuralize.command.payload
        );
        same(extraction, replay);
        let structuralized = publish(
            &f.db,
            f.correlation,
            structuralize.id,
            OfferDetailsStructuralized {
                extracted: extracted.event,
                storage: storage(),
            },
        );
        finals.push(structuralized);
    }
    let threads: Vec<_> = finals
        .into_iter()
        .map(|event| {
            let listener = f.listener.clone();
            spawn(move || listener.on_structuralized(&event).unwrap())
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(f.row()["status"], "succeeded");
    let changed = f.catalog(&["different"]);
    let replay = f.listener.on_catalog(&changed).unwrap();
    assert_eq!(replay.len(), 2);
    for (first, second) in f
        .listener
        .on_catalog(&catalog)
        .unwrap()
        .into_iter()
        .zip(replay)
    {
        same(first.encode().unwrap(), second.encode().unwrap());
    }
    let row = f.row();
    assert_eq!(
        row["payload"]["catalogs"][0]["products"]
            .as_object()
            .unwrap()
            .len(),
        2
    );
    reconcile(&f.db.pool).unwrap();
    assert_eq!(f.row(), row);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn reconciliation_distinguishes_retryable_reserved_and_pending_success() {
    let f = Fixture::new();
    for (status, attempts) in [(0, 0), (4, 5), (2, 6)] {
        f.command(&f.catalog, f.initialization, status, attempts);
        reconcile(&f.db.pool).unwrap();
        assert_eq!(f.row()["status"], "in_progress");
    }
    f.command(&f.catalog, f.initialization, 4, 6);
    let success = f.catalog(&[]);
    let event = success.metadata.as_ref().unwrap().event_id;
    f.event_status(event, 0, 0);
    reconcile(&f.db.pool).unwrap();
    assert_eq!(f.row()["status"], "in_progress");
    f.listener.on_catalog(&success).unwrap();
    reconcile(&f.db.pool).unwrap();
    assert_eq!(f.row()["status"], "succeeded");
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn exhausted_commands_and_unapplied_results_close_failed_fills() {
    for exhausted_result in [false, true] {
        let f = Fixture::new();
        f.command(&f.catalog, f.initialization, 4, 6);
        if exhausted_result {
            let result = f.catalog(&["one"]);
            f.event_status(result.metadata.unwrap().event_id, 4, 6);
        }
        reconcile(&f.db.pool).unwrap();
        assert_eq!(f.row()["status"], "failed");
        assert!(
            f.row()["payload"]["catalogs"][0]["failure"]
                .as_str()
                .unwrap()
                .contains(if exhausted_result {
                    "result delivery"
                } else {
                    "command retries"
                })
        );
        let late = f.catalog(&["one"]);
        assert!(f.listener.on_catalog(&late).unwrap().is_empty());
        assert_eq!(f.row()["status"], "failed");
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn exhausted_dispatch_only_fails_missing_successors() {
    let f = Fixture::new();
    let result = f.catalog(&["one", "two"]);
    let parent = result.metadata.as_ref().unwrap().event_id;
    let commands = f.listener.on_catalog(&result).unwrap();
    let mut commands = commands.into_iter();
    f.command(&commands.next().unwrap().encode().unwrap(), parent, 0, 0);
    f.event_status(parent, 4, 6);
    reconcile(&f.db.pool).unwrap();
    let row = f.row();
    assert_eq!(row["status"], "in_progress");
    assert!(row["payload"]["catalogs"][0]["products"]["one"]["failure"].is_null());
    assert!(
        row["payload"]["catalogs"][0]["products"]["two"]["failure"]
            .as_str()
            .unwrap()
            .contains("not dispatched")
    );
    // A subscriber error cannot fail work that the saga successfully dispatched.
    let f = Fixture::new();
    let result = f.catalog(&["one"]);
    let parent = result.metadata.as_ref().unwrap().event_id;
    let command = f
        .listener
        .on_catalog(&result)
        .unwrap()
        .remove(0)
        .encode()
        .unwrap();
    f.command(&command, parent, 0, 0);
    f.event_status(parent, 4, 6);
    reconcile(&f.db.pool).unwrap();
    assert!(f.row()["payload"]["catalogs"][0]["products"]["one"]["failure"].is_null());
    f.command(&command, parent, 4, 6);
    reconcile(&f.db.pool).unwrap();
    assert_eq!(f.row()["status"], "failed");
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn thousand_product_payload_size_and_locked_transition_time() {
    let f = Fixture::new();
    let ids: Vec<_> = (0..1000)
        .map(|id| format!("gid://shopify/Product/{id}"))
        .collect();
    let result = f.catalog(&ids.iter().map(String::as_str).collect::<Vec<_>>());
    f.listener.on_catalog(&result).unwrap();
    let repository = SagaRepository::new(f.db.pool.clone());
    repository
        .transition::<ShopifyGraphQlInitialFillSaga, _>(
            f.correlation,
            RETAILER,
            None,
            |record, _| {
                for product in record.payload.catalogs[0].products.values_mut() {
                    product.fetched = Some(ResultRef {
                        event: Uuid::now_v7(),
                        object: Uuid::now_v7(),
                    });
                    product.extracted = Some(ResultRef {
                        event: Uuid::now_v7(),
                        object: Uuid::now_v7(),
                    });
                    product.structuralized = Some(ResultRef {
                        event: Uuid::now_v7(),
                        object: Uuid::now_v7(),
                    });
                }
                Ok(())
            },
        )
        .unwrap();
    let start = Instant::now();
    assert_eq!(f.listener.on_catalog(&result).unwrap().len(), 1000);
    let elapsed = start.elapsed();
    let bytes = to_string(&f.row()["payload"]).unwrap().len();
    eprintln!(
        "1000 products with three accepted results: {bytes} JSON bytes; locked replay {elapsed:?}"
    );
    assert!(bytes < 1_000_000);
    assert_eq!(f.row()["status"], "succeeded");
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn acquired_token_and_context_snapshot_survive_changed_duplicate_results() {
    use crate::steps::graphql::acquire_storefront_token::StorefrontTokenAcquired;
    use retailer_sauce::prelude::StorefrontToken;
    let db = Database::new();
    let listener = Listener::new(SagaRepository::new(db.pool.clone()));
    let correlation = Uuid::now_v7();
    let initial = publish(
        &db,
        correlation,
        Uuid::now_v7(),
        RetailerFillInitialized {
            retailer_code: RetailerCode::AnkerAu,
        },
    );
    let command = listener
        .on_retailer_fill_initialized(&initial)
        .unwrap()
        .remove(0)
        .encode()
        .unwrap();
    let payload: Value = from_str(&command.command.payload).unwrap();
    let token = publish(
        &db,
        correlation,
        command.id,
        StorefrontTokenAcquired {
            retailer_code: RetailerCode::AnkerAu,
            storefront_token: Some(StorefrontToken::new("first-public-token".into()).unwrap()),
            url: payload["url"].as_str().unwrap().into(),
            acquired_at: Utc::now(),
        },
    );
    let first = listener
        .on_token(&token)
        .unwrap()
        .remove(0)
        .encode()
        .unwrap();
    let mut duplicate = token;
    duplicate.metadata.as_mut().unwrap().event_id = Uuid::now_v7();
    duplicate.event.storefront_token =
        Some(StorefrontToken::new("replacement-public-token".into()).unwrap());
    same(
        first,
        listener
            .on_token(&duplicate)
            .unwrap()
            .remove(0)
            .encode()
            .unwrap(),
    );
    duplicate.metadata.as_mut().unwrap().causation_id = Some(Uuid::now_v7());
    assert!(listener.on_token(&duplicate).unwrap().is_empty());
    duplicate.metadata.as_mut().unwrap().correlation_id = Some(Uuid::nil());
    assert!(listener.on_token(&duplicate).is_err());
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn unexpected_product_context_causation_and_retailer_do_not_change_progress() {
    let f = Fixture::new();
    let catalog = f.catalog(&["one"]);
    let command = f
        .listener
        .on_catalog(&catalog)
        .unwrap()
        .remove(0)
        .encode()
        .unwrap();
    let before = f.row();
    let mut event = f.fetched(command.id, "one");
    event.event.context.as_mut().unwrap().language = "unexpected".into();
    assert!(f.listener.on_fetched(&event).is_err());
    assert_eq!(f.row(), before);
    let mut event = f.fetched(Uuid::now_v7(), "one");
    assert!(f.listener.on_fetched(&event).unwrap().is_empty());
    event.metadata.as_mut().unwrap().causation_id = Some(command.id);
    event.event.retailer_code = RetailerCode::AnkerAu;
    assert!(f.listener.on_fetched(&event).is_err());
    assert_eq!(f.row(), before);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn saga_commit_survives_partial_dispatch_and_replay_inserts_missing_commands() {
    use super::register;
    use crate::app;
    use crate::platform::bus::Bus;
    use crate::platform::state::AppState;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use diesel::connection::SimpleConnection;
    use tokio::runtime::Runtime;
    use tower::ServiceExt;
    let f = Fixture::new();
    let result = f.catalog(&["one", "two"]);
    let event_id = result.metadata.as_ref().unwrap().event_id;
    // Commit the transition, as if the process died before dispatch began.
    let commands = f.listener.on_catalog(&result).unwrap();
    let failed_id = commands.into_iter().nth(1).unwrap().encode().unwrap().id;
    f.db.pool.get().unwrap().batch_execute(&format!("CREATE FUNCTION reject_command() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.id = '{failed_id}'::uuid THEN RAISE EXCEPTION 'fixture dispatch failure'; END IF; RETURN NEW; END $$; CREATE TRIGGER reject_command BEFORE INSERT ON command_entries FOR EACH ROW EXECUTE FUNCTION reject_command();")).unwrap();
    f.event_status(event_id, 0, 0);
    let runtime = Runtime::new().unwrap();
    let mut bus = Bus::default();
    register(&mut bus, &f.db.pool);
    let guard = runtime.enter();
    let bus = bus.start(f.db.pool.clone()).unwrap();
    drop(guard);
    let app = app(AppState {
        bus: Some(bus.clone()),
        ..Default::default()
    });
    let run = || {
        let response = runtime
            .block_on(
                app.clone().oneshot(
                    Request::post("/api/events/run")
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"limit":10}"#))
                        .unwrap(),
                ),
            )
            .unwrap();
        assert!(response.status().is_success());
        let bytes = runtime
            .block_on(to_bytes(response.into_body(), 1024 * 1024))
            .unwrap();
        from_slice::<Value>(&bytes).unwrap()
    };
    assert_eq!(run()["errors"].as_array().unwrap().len(), 1);
    assert_eq!(
        json_query(
            &mut f.db.pool.get().unwrap(),
            "SELECT to_jsonb(count(*)) AS value FROM command_entries"
        ),
        1
    );
    assert_eq!(
        f.row()["payload"]["catalogs"][0]["products"]
            .as_object()
            .unwrap()
            .len(),
        2
    );
    f.db.pool.get().unwrap().batch_execute("DROP TRIGGER reject_command ON command_entries; UPDATE event_entries SET scheduled_at=now() WHERE status=4").unwrap();
    assert_eq!(run()["errors"], json!([]));
    assert_eq!(
        json_query(
            &mut f.db.pool.get().unwrap(),
            "SELECT to_jsonb(count(*)) AS value FROM command_entries"
        ),
        2
    );
    f.event_status(event_id, 0, 0);
    run();
    assert_eq!(
        json_query(
            &mut f.db.pool.get().unwrap(),
            "SELECT to_jsonb(count(*)) AS value FROM command_entries"
        ),
        2
    );
    bus.shutdown();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn explicit_skip_is_terminal_but_unsuccessful() {
    use super::model::Skip;
    let f = Fixture::new();
    let catalog = f.catalog(&["one"]);
    f.listener.on_catalog(&catalog).unwrap();
    SagaRepository::new(f.db.pool.clone())
        .transition::<ShopifyGraphQlInitialFillSaga, _>(
            f.correlation,
            RETAILER,
            None,
            |record, _| {
                record.payload.catalogs[0]
                    .products
                    .get_mut("one")
                    .unwrap()
                    .skipped = Some(Skip {
                    reason: "unpublished".into(),
                    evidence: "fixture: authoritative product-unpublished observation".into(),
                });
                record.status = record.payload.status().into();
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(f.row()["status"], "failed");
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn pending_duplicate_result_can_recover_exhausted_partial_dispatch() {
    let f = Fixture::new();
    let result = f.catalog(&["one"]);
    f.listener.on_catalog(&result).unwrap();
    f.event_status(result.metadata.unwrap().event_id, 4, 6);
    let duplicate = f.catalog(&["one"]);
    let duplicate_id = duplicate.metadata.unwrap().event_id;
    f.event_status(duplicate_id, 0, 0);
    reconcile(&f.db.pool).unwrap();
    assert_eq!(f.row()["status"], "in_progress");
    assert!(f.row()["payload"]["catalogs"][0]["products"]["one"]["failure"].is_null());
    f.event_status(duplicate_id, 4, 6);
    reconcile(&f.db.pool).unwrap();
    assert_eq!(f.row()["status"], "failed");
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn removing_canonical_events_breaks_replay_and_rolls_back_the_transition() {
    let f = Fixture::new();
    let catalog = f.catalog(&["one"]);
    let command = f
        .listener
        .on_catalog(&catalog)
        .unwrap()
        .remove(0)
        .encode()
        .unwrap();
    let fetched = f.fetched(command.id, "one");
    f.listener.on_fetched(&fetched).unwrap();
    let before = f.row();
    // Deliberate violation of the retention contract, isolated to this fixture.
    sql_query("DELETE FROM event_entries WHERE id=$1")
        .bind::<SqlUuid, _>(fetched.metadata.as_ref().unwrap().event_id)
        .execute(&mut f.db.pool.get().unwrap())
        .unwrap();
    assert!(f.listener.on_fetched(&fetched).is_err());
    assert_eq!(f.row(), before);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn requested_context_identity_survives_identical_resolved_contexts() {
    use super::model::Catalog;
    use std::collections::BTreeMap;
    let f = Fixture::new();
    let requested = RequestedContext {
        country: Some("DE".into()),
        language: "EN".into(),
        observed_country: None,
        currency: None,
    };
    SagaRepository::new(f.db.pool.clone())
        .transition::<ShopifyGraphQlInitialFillSaga, _>(
            f.correlation,
            RETAILER,
            None,
            |record, _| {
                record.payload.catalogs[0].context = RequestedContext {
                    country: None,
                    language: "EN".into(),
                    observed_country: None,
                    currency: None,
                };
                record.payload.catalogs.push(Catalog {
                    context: requested.clone(),
                    resolved: None,
                    event: None,
                    products: BTreeMap::new(),
                    failure: None,
                });
                Ok(())
            },
        )
        .unwrap();
    let initial = publish(
        &f.db,
        f.correlation,
        Uuid::now_v7(),
        RetailerFillInitialized {
            retailer_code: RETAILER,
        },
    );
    let catalogs = f.listener.on_retailer_fill_initialized(&initial).unwrap();
    let mut detail_ids = vec![];
    for catalog in catalogs {
        let mut event = f.catalog(&["same-product"]);
        event.metadata.as_mut().unwrap().causation_id = Some(catalog.encode().unwrap().id);
        event.event.context = Some(requested.clone());
        detail_ids.push(
            f.listener
                .on_catalog(&event)
                .unwrap()
                .remove(0)
                .encode()
                .unwrap()
                .id,
        );
    }
    assert_eq!(detail_ids.len(), 2);
    assert_ne!(detail_ids[0], detail_ids[1]);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn manual_batches_surface_observer_errors_and_allow_retry() {
    use crate::app;
    use crate::platform::bus::Bus;
    use crate::platform::state::AppState;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use tokio::runtime::Runtime;
    use tower::ServiceExt;
    let db = Database::new();
    let runtime = Runtime::new().unwrap();
    let failing = Arc::new(AtomicBool::new(true));
    let flag = failing.clone();
    let mut bus = Bus::default();
    bus.after_batch(move || {
        if flag.load(Ordering::SeqCst) {
            Err("internal diagnostic must stay in logs".into())
        } else {
            Ok(())
        }
    });
    let guard = runtime.enter();
    let bus = bus.start(db.pool.clone()).unwrap();
    drop(guard);
    let app = app(AppState {
        bus: Some(bus.clone()),
        ..Default::default()
    });
    for path in ["/api/commands/run", "/api/events/run"] {
        for fails in [true, false] {
            failing.store(fails, Ordering::SeqCst);
            let response = runtime
                .block_on(
                    app.clone().oneshot(
                        Request::post(path)
                            .header("content-type", "application/json")
                            .body(Body::from(r#"{"limit":1}"#))
                            .unwrap(),
                    ),
                )
                .unwrap();
            assert_eq!(
                response.status(),
                if fails {
                    StatusCode::INTERNAL_SERVER_ERROR
                } else {
                    StatusCode::OK
                }
            );
            let bytes = runtime
                .block_on(to_bytes(response.into_body(), 1024))
                .unwrap();
            assert!(!String::from_utf8_lossy(&bytes).contains("internal diagnostic"));
        }
    }
    bus.shutdown();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn completed_command_retention_prevents_reexecution_after_replay() {
    use crate::platform::bus::Bus;
    use kernel::prelude::{NewCommandEnvelope, NewCommandMetadata};
    use tokio::runtime::Runtime;
    let f = Fixture::new();
    let catalog = f.catalog(&["one"]);
    let command = f
        .listener
        .on_catalog(&catalog)
        .unwrap()
        .remove(0)
        .encode()
        .unwrap();
    let parent = catalog.metadata.as_ref().unwrap().event_id;
    f.command(&command, parent, 5, 1);
    let runtime = Runtime::new().unwrap();
    let guard = runtime.enter();
    let bus = Bus::default().start(f.db.pool.clone()).unwrap();
    drop(guard);
    let dispatch = || {
        let replay = f
            .listener
            .on_catalog(&catalog)
            .unwrap()
            .remove(0)
            .encode()
            .unwrap();
        bus.command_gateway()
            .dispatch(NewCommandEnvelope {
                command: replay.command,
                metadata: Some(NewCommandMetadata {
                    command_id: replay.id,
                    correlation_id: Some(f.correlation),
                    causation_id: Some(parent),
                    source: Some("replay-test".into()),
                }),
            })
            .unwrap();
    };
    dispatch();
    assert_eq!(
        json_query(
            &mut f.db.pool.get().unwrap(),
            "SELECT to_jsonb(status) AS value FROM command_entries"
        ),
        5
    );
    // Deliberate violation of the retention contract demonstrates why cleanup is forbidden.
    sql_query("DELETE FROM command_entries WHERE id=$1")
        .bind::<SqlUuid, _>(command.id)
        .execute(&mut f.db.pool.get().unwrap())
        .unwrap();
    dispatch();
    assert_eq!(
        json_query(
            &mut f.db.pool.get().unwrap(),
            "SELECT to_jsonb(status) AS value FROM command_entries"
        ),
        0
    );
    bus.shutdown();
}
