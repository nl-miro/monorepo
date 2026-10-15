use super::entity::RetrievedSitemapDocument;
use super::handler::Store;
use super::store::{PgStore, StoreError};
use crate::platform::test_db::{Database, json_query};
use diesel::connection::SimpleConnection;
use serde_json::json;
use sitemap::RawSitemapDocument;
use uuid::Uuid;

#[test]
#[ignore = "requires TEST_DATABASE_URL; run make test-postgres"]
fn postgres_retrieved_entities_store_as_an_atomic_batch() {
    let db = Database::new();
    let store = PgStore::new(db.pool.clone());
    let mut conn = db.pool.get().unwrap();
    let retrieval_id = Uuid::now_v7();
    let docs = ["first ž", "second 🔎"]
        .into_iter()
        .map(|body| RawSitemapDocument {
            url: format!("https://shop.test/{body}"),
            last_modified: None,
            body_size: body.len(),
            body: body.into(),
        })
        .collect();
    let mut entities = RetrievedSitemapDocument::from_documents(retrieval_id, docs);
    store.store_documents(&[]).unwrap();
    conn.batch_execute("ALTER TABLE raw_sitemap_documents_data ADD CONSTRAINT reject_payload CHECK (body <> 'second 🔎')").unwrap();
    assert!(store.store_documents(&entities).is_err());
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT jsonb_build_array((SELECT count(*) FROM sitemaps_raw),(SELECT count(*) FROM raw_sitemap_documents_data)) AS value"
        ),
        json!([0, 0])
    );
    conn.batch_execute("ALTER TABLE raw_sitemap_documents_data DROP CONSTRAINT reject_payload")
        .unwrap();
    let size = entities[1].document.body_size;
    entities[1].document.body_size = i32::MAX as usize + 1;
    assert!(matches!(
        store.store_documents(&entities),
        Err(StoreError::BodyTooLarge(_))
    ));
    entities[1].document.body_size = size;
    store.store_documents(&entities).unwrap();
    let rows = json_query(
        &mut conn,
        "SELECT jsonb_agg(to_jsonb(r) || jsonb_build_object('body',d.body) ORDER BY r.id) AS value FROM sitemaps_raw r JOIN raw_sitemap_documents_data d USING(id)",
    );
    assert_eq!(rows.as_array().unwrap().len(), 2);
    for entity in &entities {
        let row = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == entity.id.to_string())
            .unwrap();
        assert_eq!(row["retrieval_id"], retrieval_id.to_string());
        assert_eq!(row["url"], entity.document.url);
        assert_eq!(row["body"], entity.document.body);
        assert_eq!(row["body_size"], entity.document.body_size);
    }
    assert!(store.store_documents(&entities).is_err());
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM raw_sitemap_documents_data"
        ),
        json!(2)
    );
}
