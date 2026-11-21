//! Coordinate stale reservation recovery with active manual batches.
use super::db::DbPool;
use chrono::Duration;
use commanding::prelude::{CommandSweepPort, DieselPgCommandRepository, StaleCommandSpec};
use diesel::sql_types::{Bool, Integer, Text};
use diesel::{Connection, PgConnection, QueryableByName, RunQueryDsl, sql_query};
use eventing::prelude::{DieselPgEventRepository, EventSweepPort, StaleEventSpec};

#[derive(Clone)]
pub(crate) struct Recovery {
    pool: DbPool,
    url: String,
}
#[derive(QueryableByName)]
struct Identity {
    #[diesel(sql_type = Text)]
    path: String,
    #[diesel(sql_type = Integer)]
    key: i32,
}
#[derive(QueryableByName)]
struct Acquired {
    #[diesel(sql_type = Bool)]
    acquired: bool,
}
// A dedicated connection owns session locks and releases them on drop, including panic.
pub(crate) struct BatchGuard(PgConnection);
impl Recovery {
    pub fn new(pool: DbPool, url: String) -> Self {
        Self { pool, url }
    }
    fn connect(&self) -> Result<(PgConnection, i32), String> {
        let identity: Identity = sql_query("SELECT current_setting('search_path') AS path, 'command_entries'::regclass::oid::integer AS key")
            .get_result(&mut self.pool.get().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let mut conn = PgConnection::establish(&self.url).map_err(|e| e.to_string())?;
        sql_query("SELECT set_config('search_path', $1, false)")
            .bind::<Text, _>(identity.path)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok((conn, identity.key))
    }
    fn sweep_on(&self, conn: &mut PgConnection, key: i32) -> Result<(), String> {
        let acquired: Acquired =
            sql_query("SELECT pg_try_advisory_lock(1396787009, $1) AS acquired")
                .bind::<Integer, _>(key)
                .get_result(conn)
                .map_err(|e| e.to_string())?;
        if !acquired.acquired {
            return Ok(());
        }
        // On failure the dedicated connection drops, releasing the exclusive lock.
        DieselPgCommandRepository::new(self.pool.clone())
            .sweep(&StaleCommandSpec::new(Duration::minutes(5)))
            .map_err(|e| e.to_string())?;
        DieselPgEventRepository::new(self.pool.clone())
            .sweep(&StaleEventSpec::new(Duration::minutes(5)))
            .map_err(|e| e.to_string())?;
        sql_query("SELECT pg_advisory_unlock(1396787009, $1)")
            .bind::<Integer, _>(key)
            .execute(conn)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn sweep(&self) -> Result<(), String> {
        let (mut conn, key) = self.connect()?;
        self.sweep_on(&mut conn, key)
    }
    pub fn begin(&self) -> Result<BatchGuard, String> {
        let (mut conn, key) = self.connect()?;
        self.sweep_on(&mut conn, key)?;
        sql_query("SELECT pg_advisory_lock_shared(1396787009, $1)")
            .bind::<Integer, _>(key)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(BatchGuard(conn))
    }
}
impl BatchGuard {
    pub fn finish(self) {
        drop(self.0);
    }
}

#[cfg(test)]
mod postgres_tests {
    use super::Recovery;
    use crate::platform::bus::{Command, Event, Handle};
    use crate::platform::test_db::{Database, json_query};
    use diesel::{RunQueryDsl, sql_query};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::env::var;
    use std::sync::{
        Mutex,
        mpsc::{Receiver, Sender},
    };
    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn sweep_waits_for_all_batches_and_preserves_recent_reservations() {
        let db = Database::new();
        let recovery = Recovery::new(db.pool.clone(), var("DATABASE_URL_TEST").unwrap());
        let first = recovery.begin().unwrap();
        let second = recovery.begin().unwrap();
        sql_query("INSERT INTO command_entries (id, command_type, payload, status, attempts, reserved_at) VALUES (gen_random_uuid(), 'test', '{}', 2, 1, now() - interval '6 minutes'), (gen_random_uuid(), 'test', '{}', 2, 1, now())").execute(&mut db.pool.get().unwrap()).unwrap();
        sql_query("INSERT INTO event_entries (id, event_type, payload, status, attempts, reserved_at) VALUES (gen_random_uuid(), 'test', '{}', 2, 6, now() - interval '6 minutes')").execute(&mut db.pool.get().unwrap()).unwrap();
        recovery.sweep().unwrap();
        let statuses = || {
            json_query(
                &mut db.pool.get().unwrap(),
                "SELECT jsonb_agg(status ORDER BY reserved_at NULLS FIRST) AS value FROM command_entries",
            )
        };
        assert_eq!(statuses(), json!([2, 2]));
        first.finish();
        recovery.sweep().unwrap();
        assert_eq!(statuses(), json!([2, 2]));
        second.finish();
        recovery.sweep().unwrap();
        assert_ne!(statuses(), json!([2, 2]));
        assert_eq!(
            json_query(
                &mut db.pool.get().unwrap(),
                "SELECT to_jsonb(status) AS value FROM event_entries"
            ),
            json!(4)
        );
        sql_query("UPDATE command_entries SET reserved_at = now() - interval '6 minutes' WHERE status = 2").execute(&mut db.pool.get().unwrap()).unwrap();
        recovery.sweep().unwrap();
        assert_eq!(
            json_query(
                &mut db.pool.get().unwrap(),
                "SELECT to_jsonb(count(*)) AS value FROM command_entries WHERE status = 2"
            ),
            json!(0)
        );
    }
    #[derive(Serialize, Deserialize)]
    struct Slow;
    impl Command for Slow {
        const NAME: &'static str = "Slow";
    }
    #[derive(Serialize, Deserialize)]
    struct Done;
    impl Event for Done {
        const NAME: &'static str = "Done";
    }
    struct Handler {
        started: Sender<()>,
        release: Mutex<Receiver<()>>,
    }
    impl Handle for Handler {
        type Command = Slow;
        type Event = Done;
        type Error = String;
        fn handle(&self, _: Slow) -> Result<Vec<Done>, String> {
            self.started.send(()).unwrap();
            self.release.lock().unwrap().recv().unwrap();
            Ok(vec![Done])
        }
    }
    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn cancelled_waiter_does_not_release_the_running_batch_guard() {
        use crate::platform::bus::Bus;
        use std::sync::mpsc::channel;
        use std::time::Duration;
        use tokio::runtime::Runtime;
        let db = Database::new();
        let runtime = Runtime::new().unwrap();
        let recovery = Recovery::new(db.pool.clone(), var("DATABASE_URL_TEST").unwrap());
        let (started, entered) = channel();
        let (release, receiver) = channel();
        let (finished, done) = channel();
        let mut bus = Bus::default();
        bus.handle(Handler {
            started,
            release: Mutex::new(receiver),
        });
        bus.after_batch(move || {
            finished.send(()).unwrap();
            Ok(())
        });
        let guard = runtime.enter();
        let mut bus = bus.start(db.pool.clone()).unwrap();
        drop(guard);
        bus.recovery = Some(recovery.clone());
        sql_query("INSERT INTO command_entries (id, command_type, payload, status) VALUES (gen_random_uuid(), 'Slow', 'null', 0)").execute(&mut db.pool.get().unwrap()).unwrap();
        let worker = bus.clone();
        let waiter = runtime.spawn(async move { worker.run_commands(1).await });
        entered.recv_timeout(Duration::from_secs(5)).unwrap();
        waiter.abort();
        sql_query("UPDATE command_entries SET reserved_at=now()-interval '6 minutes'")
            .execute(&mut db.pool.get().unwrap())
            .unwrap();
        recovery.sweep().unwrap();
        assert_eq!(
            json_query(
                &mut db.pool.get().unwrap(),
                "SELECT to_jsonb(status) AS value FROM command_entries"
            ),
            json!(2)
        );
        release.send(()).unwrap();
        done.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(
            json_query(
                &mut db.pool.get().unwrap(),
                "SELECT to_jsonb(status) AS value FROM command_entries"
            ),
            json!(5)
        );
        bus.shutdown();
    }
    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn correlation_indexes_round_trip() {
        use diesel::{PgConnection, connection::SimpleConnection};
        let db = Database::new();
        let mut conn = db.pool.get().unwrap();
        let count = |conn: &mut PgConnection| {
            json_query(
                conn,
                "SELECT to_jsonb(count(*)) AS value FROM pg_indexes WHERE schemaname=current_schema() AND indexname IN ('command_entries_correlation_idx','event_entries_correlation_idx')",
            )
        };
        assert_eq!(count(&mut conn), json!(2));
        conn.batch_execute(include_str!(
            "../../migrations/2026-11-20-000001_queue_correlation_indexes/down.sql"
        ))
        .unwrap();
        assert_eq!(count(&mut conn), json!(0));
        conn.batch_execute(include_str!(
            "../../migrations/2026-11-20-000001_queue_correlation_indexes/up.sql"
        ))
        .unwrap();
        assert_eq!(count(&mut conn), json!(2));
    }
}
