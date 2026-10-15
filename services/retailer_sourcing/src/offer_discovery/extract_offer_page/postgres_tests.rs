use super::entity::ExtractedOffer;
use super::handler::Store;
use super::store::PgStore;
use crate::platform::test_db::{Database, json_query};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Uuid as SqlUuid};
use serde_json::json;
use uuid::Uuid;

#[test]
#[ignore = "requires TEST_DATABASE_URL; run make test-postgres"]
fn postgres_entity_reads_and_atomic_upserts() {
    let db = Database::new();
    let store = PgStore::new(db.pool.clone());
    let mut conn = db.pool.get().unwrap();
    let offer_id = Uuid::now_v7();
    assert!(store.raw_offer(offer_id).unwrap().is_none());
    let input_id = Uuid::now_v7();
    diesel::sql_query("INSERT INTO offer_details_raw (id,offer_id,url,body_size,retailer_code) VALUES ($1,$2,'https://shop.test/a',1,'ankerau')")
        .bind::<SqlUuid,_>(input_id).bind::<SqlUuid,_>(offer_id).execute(&mut conn).unwrap();
    let input_value = json!("<html>ž 🔎</html>");
    diesel::sql_query("INSERT INTO raw_offers_data (id,body) VALUES ($1,($2 #>> '{}'))")
        .bind::<SqlUuid, _>(input_id)
        .bind::<Jsonb, _>(&input_value)
        .execute(&mut conn)
        .unwrap();
    let input = store.raw_offer(offer_id).unwrap().unwrap();
    assert_eq!(input.offer_id, offer_id);
    assert_eq!(json!(input.body), input_value);
    assert!(store.raw_offer(Uuid::now_v7()).unwrap().is_none());

    let mut entity = ExtractedOffer {
        offer_id,
        document: json!({"value":"first ž"}),
    };
    conn.batch_execute(
        "ALTER TABLE extracted_offers_data ADD CONSTRAINT reject_payload CHECK (false)",
    )
    .unwrap();
    assert!(store.store_extracted_offer(&entity).is_err());
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM offer_details_extracted"
        ),
        json!(0)
    );
    conn.batch_execute("ALTER TABLE extracted_offers_data DROP CONSTRAINT reject_payload")
        .unwrap();
    store.store_extracted_offer(&entity).unwrap();
    let original = json_query(
        &mut conn,
        "SELECT to_jsonb(o) || jsonb_build_object('document',d.document) AS value FROM offer_details_extracted o JOIN extracted_offers_data d USING(id)",
    );
    assert_eq!(original["document"], entity.document);
    entity.document = json!({"value":"updated 🔎"});
    store.store_extracted_offer(&entity).unwrap();
    let updated = json_query(
        &mut conn,
        "SELECT to_jsonb(o) || jsonb_build_object('document',d.document) AS value FROM offer_details_extracted o JOIN extracted_offers_data d USING(id)",
    );
    assert_eq!(updated["id"], original["id"]);
    assert_eq!(updated["offer_id"], offer_id.to_string());
    assert_eq!(updated["document"], entity.document);
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT jsonb_build_array((SELECT count(*) FROM offer_details_extracted),(SELECT count(*) FROM extracted_offers_data)) AS value"
        ),
        json!([1, 1])
    );
    conn.batch_execute("ALTER TABLE extracted_offers_data ADD CONSTRAINT reject_update CHECK (document->>'value' <> 'rejected')").unwrap();
    entity.document = json!({"value":"rejected"});
    assert!(store.store_extracted_offer(&entity).is_err());
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(o) || jsonb_build_object('document',d.document) AS value FROM offer_details_extracted o JOIN extracted_offers_data d USING(id)"
        ),
        updated
    );
}
