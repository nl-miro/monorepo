use super::handler::Store;
use super::store::PgStore;
use super::store::StoreError;
use crate::platform::test_db::Database;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Uuid as SqlUuid};
use serde_json::json;
use uuid::Uuid;

#[test]
#[ignore = "requires TEST_DATABASE_URL; run make test-postgres"]
fn postgres_discovery_returns_entities_and_rejects_bad_content() {
    let db = Database::new();
    let store = PgStore::new(db.pool.clone());
    let mut conn = db.pool.get().unwrap();
    let id = Uuid::now_v7();
    assert!(matches!(
        store.consolidated_sitemap(id),
        Err(StoreError::Database(diesel::result::Error::NotFound))
    ));
    diesel::sql_query("INSERT INTO sitemaps_consolidated (id,stats,\"references\",storage) VALUES ($1,'{}','{}','{}')")
        .bind::<SqlUuid,_>(id).execute(&mut conn).unwrap();
    diesel::sql_query("INSERT INTO consolidated_sitemaps_data (id,content) VALUES ($1,$2)")
        .bind::<SqlUuid, _>(id)
        .bind::<Jsonb, _>(json!({"product":["https://shop.test/ž"],"unknown":[]}))
        .execute(&mut conn)
        .unwrap();
    let entity = store.consolidated_sitemap(id).unwrap();
    assert_eq!(entity.id, id);
    assert_eq!(entity.product_urls, ["https://shop.test/ž"]);
    conn.batch_execute("UPDATE consolidated_sitemaps_data SET content='{\"product\":[]}'")
        .unwrap();
    assert!(
        store
            .consolidated_sitemap(id)
            .unwrap()
            .product_urls
            .is_empty()
    );
    conn.batch_execute("UPDATE consolidated_sitemaps_data SET content='{\"product\":42}'")
        .unwrap();
    assert!(matches!(
        store.consolidated_sitemap(id),
        Err(StoreError::Content(_))
    ));
}
