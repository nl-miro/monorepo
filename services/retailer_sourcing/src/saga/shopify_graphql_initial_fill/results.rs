//! Loads immutable canonical result events used to rebuild downstream commands.
use crate::platform::bus::Event;
use diesel::sql_types::{Text, Uuid as SqlUuid};
use diesel::{PgConnection, QueryableByName, RunQueryDsl, sql_query};
use serde::de::DeserializeOwned;
use serde_json::from_str;
use uuid::Uuid;

#[derive(QueryableByName)]
struct StoredResult {
    #[diesel(sql_type = Text)]
    payload: String,
}

pub(super) fn load_result<E: Event + DeserializeOwned>(
    conn: &mut PgConnection,
    event_id: Uuid,
) -> Result<E, String> {
    let row = sql_query("SELECT payload FROM event_entries WHERE id = $1 AND event_type = $2")
        .bind::<SqlUuid, _>(event_id)
        .bind::<Text, _>(E::NAME)
        .get_result::<StoredResult>(conn)
        .map_err(|error| error.to_string())?;
    from_str(&row.payload).map_err(|error| error.to_string())
}
