use super::entity::ProcessedSitemap;
use super::handler::Store;
use super::store::PgStore;
use super::store::StoreError;
use crate::platform::test_db::{Database, json_query};
use bodul_shared::retailer::RetailerCode;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Uuid as SqlUuid};
use serde_json::json;
use sitemap::SitemapDocument;
use uuid::Uuid;

#[test]
#[ignore = "requires TEST_DATABASE_URL; run make test-postgres"]
fn postgres_processing_reads_entities_and_writes_atomically() {
    let db = Database::new();
    let store = PgStore::new(db.pool.clone());
    let mut conn = db.pool.get().unwrap();
    let retrieval_id = Uuid::now_v7();
    assert!(store.raw_documents(retrieval_id).unwrap().is_empty());
    for (second, body) in [(2, "second 🔎"), (1, "first ž")] {
        let id = Uuid::now_v7();
        diesel::sql_query(format!("INSERT INTO sitemaps_raw (id,retrieval_id,url,body_size,fetched_at) VALUES ($1,$2,'https://shop.test/{second}',{},'2026-10-15T00:00:0{second}Z')",body.len()))
            .bind::<SqlUuid,_>(id).bind::<SqlUuid,_>(retrieval_id).execute(&mut conn).unwrap();
        diesel::sql_query(
            "INSERT INTO raw_sitemap_documents_data (id,body) VALUES ($1,($2 #>> '{}'))",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Jsonb, _>(json!(body))
        .execute(&mut conn)
        .unwrap();
    }
    let entities = store.raw_documents(retrieval_id).unwrap();
    assert_eq!(
        entities.iter().map(|e| e.body.as_str()).collect::<Vec<_>>(),
        ["first ž", "second 🔎"]
    );
    assert_eq!(entities[0].body_size, "first ž".len());
    assert!(store.raw_documents(Uuid::now_v7()).unwrap().is_empty());
    conn.batch_execute("UPDATE sitemaps_raw SET body_size=-1")
        .unwrap();
    assert!(matches!(
        store.raw_documents(retrieval_id),
        Err(StoreError::InvalidBodySize(-1))
    ));
    let entity = ProcessedSitemap::new(
        retrieval_id,
        RetailerCode::UgreenCa,
        SitemapDocument::default(),
    );
    conn.batch_execute(
        "ALTER TABLE processed_sitemaps_data ADD CONSTRAINT reject_payload CHECK (false)",
    )
    .unwrap();
    assert!(store.store_processed(&entity).is_err());
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM sitemaps_processed"
        ),
        json!(0)
    );
    conn.batch_execute("ALTER TABLE processed_sitemaps_data DROP CONSTRAINT reject_payload")
        .unwrap();
    store.store_processed(&entity).unwrap();
    let row = json_query(
        &mut conn,
        "SELECT to_jsonb(p) || jsonb_build_object('document',d.document) AS value FROM sitemaps_processed p JOIN processed_sitemaps_data d USING(id)",
    );
    assert_eq!(row["id"], entity.id.to_string());
    assert_eq!(row["retrieval_id"], retrieval_id.to_string());
    assert_eq!(row["url_count"], entity.url_count());
    assert_eq!(row["retailer_code"], "ugreenca");
    assert_eq!(
        row["document"],
        serde_json::to_value(entity.document()).unwrap()
    );
    assert_eq!(row["document_size"], row["document"].to_string().len());
}
