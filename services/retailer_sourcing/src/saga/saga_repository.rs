//! Transactional storage for typed saga records.

use super::saga::{Saga, SagaRecord};
use super::schema::sagas::dsl::{
    correlation_id, id, payload, retailer_code, saga_type, sagas, status,
};
use crate::platform::db::{DbPool, Json};
use diesel::r2d2::PoolError;
use diesel::result::Error as DieselError;
use diesel::{
    Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl,
    insert_into, update,
};
use shared::retailer::RetailerCode;
use thiserror::Error;
use uuid::{Uuid, Variant};

#[derive(Clone)]
pub(crate) struct SagaRepository {
    pool: DbPool,
}

#[derive(Debug, Error)]
pub(crate) enum SaveError {
    #[error("could not acquire a database connection: {0}")]
    Connection(#[from] PoolError),
    #[error("could not save the saga: {0}")]
    Database(#[from] DieselError),
    #[error("invalid saga: {0}")]
    Invalid(String),
}

fn validate_uuid(value: Uuid) -> Result<(), SaveError> {
    if value.get_version_num() != 7 || value.get_variant() != Variant::RFC4122 {
        return Err(SaveError::Invalid("identity must be an RFC UUID v7".into()));
    }
    Ok(())
}

fn validate<S: Saga>(record: &SagaRecord<S>) -> Result<(), SaveError> {
    validate_uuid(record.id)?;
    validate_uuid(record.correlation_id)?;
    if !matches!(
        record.status.as_str(),
        "in_progress" | "succeeded" | "failed"
    ) {
        return Err(SaveError::Invalid("unknown status".into()));
    }
    Ok(())
}

impl SagaRepository {
    pub(crate) fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    #[cfg(test)]
    pub(super) fn find<S: Saga>(
        &self,
        saga_id: Uuid,
        retailer: RetailerCode,
    ) -> Result<Option<SagaRecord<S>>, String> {
        validate_uuid(saga_id).map_err(|error| error.to_string())?;
        let mut conn = self.pool.get().map_err(|error| error.to_string())?;
        let stored = sagas
            .find(saga_id)
            .filter(saga_type.eq(S::TYPE))
            .select((correlation_id, status, retailer_code, payload))
            .first::<(Uuid, String, String, Json<S>)>(&mut conn)
            .optional()
            .map_err(|error| error.to_string())?;
        stored
            .map(|(correlation, saved_status, retailer_name, Json(state))| {
                if retailer_name != retailer.as_str() {
                    return Err("saga retailer mismatch".into());
                }
                let record = SagaRecord {
                    id: saga_id,
                    correlation_id: correlation,
                    status: saved_status,
                    retailer_code: retailer,
                    payload: state,
                };
                validate(&record).map_err(|error| error.to_string())?;
                Ok(record)
            })
            .transpose()
    }

    fn save_on<S: Saga>(conn: &mut PgConnection, record: &SagaRecord<S>) -> Result<(), SaveError> {
        let changed = update(
            sagas
                .find(record.id)
                .filter(saga_type.eq(S::TYPE))
                .filter(correlation_id.eq(record.correlation_id))
                .filter(retailer_code.eq(record.retailer_code.as_str())),
        )
        .set((payload.eq(Json(&record.payload)), status.eq(&record.status)))
        .execute(conn)?;
        if changed != 1 {
            return Err(SaveError::Invalid(
                "record identity or retailer mismatch".into(),
            ));
        }
        Ok(())
    }

    fn insert_initial<S: Saga>(
        conn: &mut PgConnection,
        correlation: Uuid,
        retailer: RetailerCode,
        initial: S,
    ) -> Result<usize, SaveError> {
        Ok(insert_into(sagas)
            .values((
                id.eq(Uuid::now_v7()),
                saga_type.eq(S::TYPE),
                correlation_id.eq(correlation),
                retailer_code.eq(retailer.as_str()),
                status.eq("in_progress"),
                payload.eq(Json(initial)),
            ))
            .on_conflict((saga_type, correlation_id))
            .do_nothing()
            .execute(conn)?)
    }

    /// Replays verify the saved owner before consulting live inventory. Concurrent
    /// first attempts converge on the unique type/correlation pair.
    pub(crate) fn initialize<S: Saga>(
        &self,
        correlation: Uuid,
        retailer: RetailerCode,
        snapshot: impl FnOnce() -> Result<S, String>,
    ) -> Result<(), SaveError> {
        validate_uuid(correlation)?;
        let mut conn = self.pool.get()?;
        conn.transaction(|conn| {
            let owner = || {
                sagas
                    .filter(saga_type.eq(S::TYPE))
                    .filter(correlation_id.eq(correlation))
                    .select(retailer_code)
            };
            let saved = match owner().first::<String>(conn).optional()? {
                Some(saved) => saved,
                None => {
                    let initial = snapshot().map_err(SaveError::Invalid)?;
                    if Self::insert_initial(conn, correlation, retailer, initial)? == 1 {
                        return Ok(());
                    }
                    owner().first::<String>(conn)?
                }
            };
            if saved != retailer.as_str() {
                return Err(SaveError::Invalid("saga retailer mismatch".into()));
            }
            Ok(())
        })
    }

    /// Create or load under the correlation constraint, then apply and save with
    /// the same connection and row lock. A failed callback rolls back all writes.
    pub(crate) fn transition<S: Saga, R>(
        &self,
        correlation: Uuid,
        retailer: RetailerCode,
        initial: Option<S>,
        apply: impl FnOnce(&mut SagaRecord<S>, &mut PgConnection) -> Result<R, SaveError>,
    ) -> Result<Option<R>, SaveError> {
        validate_uuid(correlation)?;
        let mut conn = self.pool.get()?;
        conn.transaction::<_, SaveError, _>(|conn| {
            if let Some(initial) = initial {
                Self::insert_initial(conn, correlation, retailer, initial)?;
            }
            let stored = sagas
                .filter(saga_type.eq(S::TYPE))
                .filter(correlation_id.eq(correlation))
                .select((id, retailer_code, status, payload))
                .for_update()
                .first::<(Uuid, String, String, Json<S>)>(conn)
                .optional()?;
            let Some((saved_id, retailer_name, saved_status, Json(state))) = stored else {
                return Ok(None);
            };
            if retailer_name != retailer.as_str() {
                return Err(SaveError::Invalid("saga retailer mismatch".into()));
            }
            let mut record = SagaRecord {
                id: saved_id,
                correlation_id: correlation,
                status: saved_status,
                retailer_code: retailer,
                payload: state,
            };
            validate(&record)?;
            let previous_status = record.status.clone();
            let result = apply(&mut record, conn)?;
            validate(&record)?;
            if record.id != saved_id
                || record.correlation_id != correlation
                || record.retailer_code != retailer
            {
                return Err(SaveError::Invalid(
                    "transition changed record identity".into(),
                ));
            }
            if previous_status != "in_progress" && record.status != previous_status {
                return Err(SaveError::Invalid(
                    "terminal saga cannot change status".into(),
                ));
            }
            Self::save_on(conn, &record)?;
            Ok(Some(result))
        })
    }
}

#[cfg(test)]
mod postgres_tests {
    use super::{SagaRepository, SaveError, validate_uuid};
    use crate::platform::test_db::Database;
    use crate::saga::saga::Saga;
    use serde::{Deserialize, Serialize};
    use shared::retailer::RetailerCode;
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize)]
    struct State {
        count: usize,
    }
    impl Saga for State {
        const TYPE: &'static str = "StorageTest";
    }

    #[test]
    fn validates_uuid_version_and_variant() {
        assert!(validate_uuid(Uuid::now_v7()).is_ok());
        assert!(validate_uuid(Uuid::nil()).is_err());
        assert!(
            validate_uuid(Uuid::parse_str("01900000-0000-7000-0000-000000000000").unwrap())
                .is_err()
        );
    }

    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn transitions_recover_identity_and_roll_back_failures() {
        let db = Database::new();
        let repository = SagaRepository::new(db.pool.clone());
        let correlation = Uuid::now_v7();
        let retailer = RetailerCode::AnkerAu;
        let saga_id = repository
            .transition(
                correlation,
                retailer,
                Some(State { count: 0 }),
                |record, _| {
                    record.payload.count += 1;
                    Ok(record.id)
                },
            )
            .unwrap()
            .unwrap();
        repository
            .transition(
                correlation,
                retailer,
                Some(State { count: 99 }),
                |record, _| {
                    assert_eq!(record.id, saga_id);
                    assert_eq!(record.payload.count, 1);
                    record.payload.count = 2;
                    Ok(())
                },
            )
            .unwrap();
        let failure =
            repository.transition::<State, ()>(correlation, retailer, None, |record, _| {
                record.payload.count = 100;
                Err(SaveError::Invalid("injected rollback".into()))
            });
        assert!(failure.is_err());
        assert_eq!(
            repository
                .find::<State>(saga_id, retailer)
                .unwrap()
                .unwrap()
                .payload
                .count,
            2
        );
        assert!(
            repository
                .transition::<State, ()>(correlation, RetailerCode::AnkerCa, None, |_, _| Ok(()))
                .is_err()
        );
        assert!(
            repository
                .transition::<State, ()>(Uuid::now_v7(), retailer, None, |_, _| Ok(()))
                .unwrap()
                .is_none()
        );
    }
    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn concurrent_transitions_serialize_without_losing_updates() {
        use std::sync::{Arc, Barrier};
        use std::thread::spawn;
        let db = Database::new();
        let repository = SagaRepository::new(db.pool.clone());
        let correlation = Uuid::now_v7();
        let gate = Arc::new(Barrier::new(2));
        let threads: Vec<_> = (0..2)
            .map(|_| {
                let repository = repository.clone();
                let gate = gate.clone();
                spawn(move || {
                    gate.wait();
                    for _ in 0..10 {
                        repository
                            .transition(
                                correlation,
                                RetailerCode::AnkerAu,
                                Some(State { count: 0 }),
                                |record, _| {
                                    record.payload.count += 1;
                                    Ok(())
                                },
                            )
                            .unwrap();
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        repository
            .transition::<State, ()>(correlation, RetailerCode::AnkerAu, None, |record, _| {
                assert_eq!(record.payload.count, 20);
                record.status = "succeeded".into();
                Ok(())
            })
            .unwrap();
        assert!(
            repository
                .transition::<State, ()>(correlation, RetailerCode::AnkerAu, None, |record, _| {
                    record.status = "in_progress".into();
                    Ok(())
                })
                .is_err()
        );
    }

    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn database_rejects_invalid_identity_status_and_payload() {
        use diesel::sql_types::{Text, Uuid as SqlUuid};
        use diesel::{RunQueryDsl, sql_query};
        let db = Database::new();
        let mut conn = db.pool.get().unwrap();
        let valid = Uuid::now_v7();
        for (saga_id, correlation, state, body) in [
            (Uuid::nil(), valid, "in_progress", "{}"),
            (valid, Uuid::nil(), "in_progress", "{}"),
            (
                Uuid::parse_str("01900000-0000-7000-0000-000000000000").unwrap(),
                valid,
                "in_progress",
                "{}",
            ),
            (valid, valid, "completed", "{}"),
            (valid, valid, "in_progress", "[]"),
        ] {
            assert!(sql_query("INSERT INTO sagas (id, correlation_id, saga_type, retailer_code, status, payload) VALUES ($1, $2, 'StorageTest', 'ankerau', $3, $4::jsonb)")
                .bind::<SqlUuid, _>(saga_id).bind::<SqlUuid, _>(correlation)
                .bind::<Text, _>(state).bind::<Text, _>(body).execute(&mut conn).is_err());
        }
    }
}
