use super::entity::RawOffer;
use super::handler::Store;
use super::store::PgStore;
use crate::platform::test_db::{Database, json_query};
use bodul_shared::retailer::RetailerCode;
use diesel::connection::SimpleConnection;
use serde_json::json;
use uuid::Uuid;

const ROW: &str = "SELECT to_jsonb(o) || jsonb_build_object('body',d.body) AS value FROM offer_details_raw o JOIN raw_offers_data d USING(id)";
#[test]
#[ignore = "requires TEST_DATABASE_URL; run make test-postgres"]
fn postgres_download_entity_preserves_upsert_identity_and_atomicity() {
    let db = Database::new();
    let store = PgStore::new(db.pool.clone());
    let mut conn = db.pool.get().unwrap();
    let mut entity = RawOffer {
        offer_id: Uuid::now_v7(),
        retailer_code: RetailerCode::AnkerAu,
        url: "https://shop.test/a".into(),
        body: "ž 🔎".into(),
    };
    conn.batch_execute("ALTER TABLE raw_offers_data ADD CONSTRAINT reject_payload CHECK (false)")
        .unwrap();
    assert!(store.store_raw_offer(&entity).is_err());
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM offer_details_raw"
        ),
        json!(0)
    );
    conn.batch_execute("ALTER TABLE raw_offers_data DROP CONSTRAINT reject_payload")
        .unwrap();
    store.store_raw_offer(&entity).unwrap();
    let before = json_query(&mut conn, ROW);
    assert_eq!(before["body"], entity.body);
    assert_eq!(before["body_size"], entity.body.len());
    assert_eq!(before["url"], entity.url);
    entity.body = "replacement 🔎".into();
    entity.retailer_code = RetailerCode::UgreenCa;
    store.store_raw_offer(&entity).unwrap();
    let after = json_query(&mut conn, ROW);
    assert_eq!(after["id"], before["id"]);
    assert_eq!(after["offer_id"], entity.offer_id.to_string());
    assert_eq!(after["retailer_code"], "ugreenca");
    assert_eq!(after["body"], entity.body);
    assert_eq!(after["body_size"], entity.body.len());
    conn.batch_execute(
        "ALTER TABLE raw_offers_data ADD CONSTRAINT reject_update CHECK (body <> 'rejected')",
    )
    .unwrap();
    entity.body = "rejected".into();
    assert!(store.store_raw_offer(&entity).is_err());
    assert_eq!(json_query(&mut conn, ROW), after);
}
