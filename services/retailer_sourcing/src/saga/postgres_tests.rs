use super::schema::shopify_graphql_initial_fills::{payload, table};
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::update;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, from_value, json, to_value};
use uuid::Uuid;

use crate::platform::test_db::Database;
use crate::saga::prelude::{
    DieselPgSagaRepository, RecordableSaga, RepositoryError, SagaId, SagaRepository,
    ShopifyGraphQlInitialFillSaga, SitemapHtmlDetailsInitialFillSaga,
};
use crate::saga::record::{DieselPgSagaRecord, SagaRecordError, SagaTypeMismatch};

fn assert_repository_contract<S>()
where
    S: RecordableSaga
        + DeserializeOwned
        + Serialize
        + TryFrom<DieselPgSagaRecord, Error = SagaRecordError>
        + TryInto<DieselPgSagaRecord, Error = SagaTypeMismatch>,
{
    let db = Database::new();
    let writer = DieselPgSagaRepository::new(db.pool.clone());
    let reader = DieselPgSagaRepository::new(db.pool.clone());
    let id = SagaId::new();
    let original = json!({
        "saga_id": id,
        "correlation_id": SagaId::new(),
        "retailer_code": "ugreenca"
    });
    assert!(matches!(
        reader.load::<S>(id),
        Err(RepositoryError::Database(DieselError::NotFound))
    ));
    assert!(matches!(
        writer.update(from_value::<S>(original.clone()).unwrap()),
        Err(RepositoryError::Database(DieselError::NotFound))
    ));
    assert!(matches!(
        reader.load::<S>(id),
        Err(RepositoryError::Database(DieselError::NotFound))
    ));

    writer
        .insert(from_value::<S>(original.clone()).unwrap())
        .unwrap();
    assert_eq!(to_value(reader.load::<S>(id).unwrap()).unwrap(), original);
    let mut changed = original.clone();
    changed["correlation_id"] = to_value(SagaId::new()).unwrap();
    changed["retailer_code"] = json!("ugreenus");
    for duplicate in [&original, &changed] {
        assert!(matches!(
            writer.insert(from_value::<S>(duplicate.clone()).unwrap()),
            Err(RepositoryError::Database(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _
            )))
        ));
        assert_eq!(to_value(reader.load::<S>(id).unwrap()).unwrap(), original);
    }

    let other_id = SagaId::new();
    let mut other = original.clone();
    other["saga_id"] = to_value(other_id).unwrap();
    writer
        .insert(from_value::<S>(other.clone()).unwrap())
        .unwrap();
    for _ in 0..2 {
        writer
            .update(from_value::<S>(changed.clone()).unwrap())
            .unwrap();
        assert_eq!(to_value(reader.load::<S>(id).unwrap()).unwrap(), changed);
        assert_eq!(
            to_value(reader.load::<S>(other_id).unwrap()).unwrap(),
            other
        );
    }

    let row = table.find(Uuid::from(id));
    let mut conn = db.pool.get().unwrap();
    assert_eq!(
        row.select(payload).first::<Value>(&mut conn).unwrap(),
        json!({"retailer_code": "ugreenus"})
    );
    for invalid in [json!({}), json!({"retailer_code": "unknown"})] {
        update(row)
            .set(payload.eq(invalid))
            .execute(&mut conn)
            .unwrap();
        assert!(matches!(
            reader.load::<S>(id),
            Err(RepositoryError::InvalidRetailerCode(_))
        ));
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn inserts_and_updates_shopify_sagas_across_repository_instances() {
    assert_repository_contract::<ShopifyGraphQlInitialFillSaga>();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn inserts_and_updates_sitemap_sagas_across_repository_instances() {
    assert_repository_contract::<SitemapHtmlDetailsInitialFillSaga>();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn neither_saga_type_can_load_or_update_the_other() {
    let db = Database::new();
    let repository = DieselPgSagaRepository::new(db.pool.clone());
    let shopify_id = SagaId::new();
    let sitemap_id = SagaId::new();
    let shopify = json!({
        "saga_id": shopify_id,
        "correlation_id": SagaId::new(),
        "retailer_code": "ugreenca"
    });
    let sitemap = json!({
        "saga_id": sitemap_id,
        "correlation_id": SagaId::new(),
        "retailer_code": "ugreenus"
    });
    repository
        .insert(from_value::<ShopifyGraphQlInitialFillSaga>(shopify.clone()).unwrap())
        .unwrap();
    repository
        .insert(from_value::<SitemapHtmlDetailsInitialFillSaga>(sitemap.clone()).unwrap())
        .unwrap();

    assert!(matches!(
        repository.load::<SitemapHtmlDetailsInitialFillSaga>(shopify_id),
        Err(RepositoryError::SagaTypeMismatch(_))
    ));
    assert!(matches!(
        repository.load::<ShopifyGraphQlInitialFillSaga>(sitemap_id),
        Err(RepositoryError::SagaTypeMismatch(_))
    ));
    assert!(matches!(
        repository
            .update(from_value::<SitemapHtmlDetailsInitialFillSaga>(shopify.clone()).unwrap()),
        Err(RepositoryError::Database(DieselError::NotFound))
    ));
    assert!(matches!(
        repository.update(from_value::<ShopifyGraphQlInitialFillSaga>(sitemap.clone()).unwrap()),
        Err(RepositoryError::Database(DieselError::NotFound))
    ));
    assert_eq!(
        to_value(
            repository
                .load::<ShopifyGraphQlInitialFillSaga>(shopify_id)
                .unwrap()
        )
        .unwrap(),
        shopify
    );
    assert_eq!(
        to_value(
            repository
                .load::<SitemapHtmlDetailsInitialFillSaga>(sitemap_id)
                .unwrap()
        )
        .unwrap(),
        sitemap
    );
}
