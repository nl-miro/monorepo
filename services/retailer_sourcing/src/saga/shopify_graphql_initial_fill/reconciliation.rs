//! Observe exhausted queue work under the same saga lock as successful results.

use super::ShopifyGraphQlInitialFillSaga;
use crate::platform::bus::{Command, Event, stable_command_id};
use crate::platform::db::DbPool;
use crate::saga::saga::Saga;
use crate::saga::saga_repository::{SagaRepository, SaveError};
use crate::steps::graphql::acquire_storefront_token::{
    AcquireStorefrontToken, StorefrontTokenAcquired,
};
use crate::steps::graphql::extract_offer_details::{ExtractOfferDetails, OfferDetailsExtracted};
use crate::steps::graphql::offer_details::OfferDetailsFetched;
use crate::steps::graphql::product_catalog_ids::ProductCatalogIdsRetrieved;
use crate::steps::graphql::structuralize_offer_details::{
    OfferDetailsStructuralized, StructuralizeOfferDetails,
};
use commanding::prelude::{CommandStatus, ReservableCommandSpec};
use diesel::result::Error as DieselError;
use diesel::sql_types::{Bool, Integer, Nullable, Text, Uuid as SqlUuid};
use diesel::{PgConnection, QueryableByName, RunQueryDsl, sql_query};
use eventing::prelude::{EventStatus, ReservableEventSpec};
use serde_json::to_value;
use shared::retailer::RetailerCode;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

#[derive(QueryableByName)]
struct ActiveSaga {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    correlation_id: Uuid,
    #[diesel(sql_type = Text)]
    retailer_code: String,
}

/// Only active sagas need observation. Recheck status after acquiring their lock.
#[derive(Debug, Default)]
pub(crate) struct ReconciliationReport {
    pub scanned: usize,
    pub changed: usize,
    pub failed: usize,
}

pub(crate) fn reconcile(pool: &DbPool) -> Result<ReconciliationReport, SaveError> {
    let rows = sql_query("SELECT id, correlation_id, retailer_code FROM sagas WHERE saga_type = $1 AND status = 'in_progress'")
        .bind::<Text, _>(ShopifyGraphQlInitialFillSaga::TYPE)
        .load::<ActiveSaga>(&mut pool.get()?)?;
    let repository = SagaRepository::new(pool.clone());
    let mut report = ReconciliationReport::default();
    for row in rows {
        report.scanned += 1;
        let result = (|| {
            let retailer = RetailerCode::from_str(&row.retailer_code)
                .map_err(|error| SaveError::Invalid(error.to_string()))?;
            let changed = repository.transition::<ShopifyGraphQlInitialFillSaga, _>(
                row.correlation_id,
                retailer,
                None,
                |record, conn| {
                    if record.status != "in_progress" {
                        return Ok(false);
                    }
                    let before = to_value((&record.status, &record.payload))
                        .map_err(|e| SaveError::Invalid(e.to_string()))?;
                    let queue = Queue::load(conn, record.correlation_id)?;
                    record.payload.reconcile(&queue, record.id, retailer);
                    record.status = record.payload.status().into();
                    Ok(before
                        != to_value((&record.status, &record.payload))
                            .map_err(|e| SaveError::Invalid(e.to_string()))?)
                },
            )?;
            Ok(changed.unwrap_or(false))
        })();
        match result {
            Ok(changed) => report.changed += usize::from(changed),
            Err(
                error @ (SaveError::Invalid(_)
                | SaveError::Database(DieselError::DeserializationError(_))),
            ) => {
                report.failed += 1;
                error!(saga_id = %row.id, correlation_id = %row.correlation_id, saga_type = ShopifyGraphQlInitialFillSaga::TYPE, %error, "could not reconcile saga");
            }
            Err(error) => return Err(error),
        }
    }
    info!(
        scanned = report.scanned,
        changed = report.changed,
        failed = report.failed,
        "saga reconciliation finished"
    );
    Ok(report)
}

#[derive(QueryableByName)]
struct Entry {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Bool)]
    is_event: bool,
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    cause: Option<Uuid>,
    #[diesel(sql_type = Integer)]
    status: i32,
    #[diesel(sql_type = Integer)]
    attempts: i32,
    #[diesel(sql_type = Text)]
    reason: String,
}
impl Entry {
    fn exhausted(&self) -> bool {
        if self.is_event {
            self.status == EventStatus::Dead as i32
                || (self.status == EventStatus::Failed as i32
                    && self.attempts >= ReservableEventSpec::DEFAULT_MAX_ATTEMPTS)
        } else {
            self.status == CommandStatus::Dead as i32
                || (self.status == CommandStatus::Failed as i32
                    && self.attempts >= ReservableCommandSpec::DEFAULT_MAX_ATTEMPTS)
        }
    }
    fn completed(&self) -> bool {
        if self.is_event {
            self.status == EventStatus::Completed as i32
        } else {
            self.status == CommandStatus::Completed as i32
        }
    }
    fn failure(&self, description: &str) -> String {
        format!("{description}: {} {}: {}", self.kind, self.id, self.reason)
    }
}
struct Queue {
    commands: HashMap<Uuid, Entry>,
    events: HashMap<Uuid, Entry>,
    results: HashMap<(Uuid, String), Vec<Uuid>>,
}
impl Queue {
    fn load(conn: &mut PgConnection, correlation: Uuid) -> Result<Self, SaveError> {
        // One statement gives a consistent view of publication and command status.
        let rows = sql_query("SELECT id, false AS is_event, command_type AS kind, (meta->>'causation_id')::uuid AS cause, status, attempts, COALESCE(extra_info->'errors'->>-1, '') AS reason FROM command_entries WHERE meta->>'correlation_id' = $1 UNION ALL SELECT id, true AS is_event, event_type AS kind, (meta->>'causation_id')::uuid AS cause, status, attempts, COALESCE(extra_info->'errors'->>-1, '') AS reason FROM event_entries WHERE meta->>'correlation_id' = $1")
            .bind::<Text, _>(correlation.to_string()).load::<Entry>(conn)?;
        let mut queue = Self {
            commands: HashMap::new(),
            events: HashMap::new(),
            results: HashMap::new(),
        };
        for row in rows {
            if row.is_event {
                if let Some(cause) = row.cause {
                    queue
                        .results
                        .entry((cause, row.kind.clone()))
                        .or_default()
                        .push(row.id);
                }
                queue.events.insert(row.id, row);
            } else {
                queue.commands.insert(row.id, row);
            }
        }
        Ok(queue)
    }
    /// Called only for work with no accepted outcome. Pending publications and
    /// active/retryable commands may still deliver success and must win the race.
    fn failure(&self, command: Uuid, result_type: &str, trigger: Option<Uuid>) -> Option<String> {
        let results: Vec<_> = self
            .results
            .get(&(command, result_type.into()))
            .into_iter()
            .flatten()
            .map(|id| &self.events[id])
            .collect();
        if results
            .iter()
            .any(|event| !event.exhausted() && !event.completed())
        {
            return None;
        }
        if let Some(entry) = self.commands.get(&command) {
            if !entry.exhausted() && !entry.completed() {
                return None;
            }
            if let Some(event) = results.first() {
                return Some(event.failure("result delivery ended without an accepted outcome"));
            }
            return Some(entry.failure(if entry.exhausted() {
                "command retries exhausted"
            } else {
                "command completed without a result"
            }));
        }
        let trigger = trigger.and_then(|id| self.events.get(&id))?;
        // Another result from the same upstream command can still replay the
        // canonical transition and recover a partially dispatched successor.
        if let Some(cause) = trigger.cause
            && self
                .results
                .get(&(cause, trigger.kind.clone()))
                .into_iter()
                .flatten()
                .map(|id| &self.events[id])
                .any(|event| !event.exhausted() && !event.completed())
        {
            return None;
        }
        (trigger.exhausted() || trigger.completed())
            .then(|| trigger.failure(&format!("successor {command} was not dispatched")))
    }
}
impl ShopifyGraphQlInitialFillSaga {
    fn reconcile(&mut self, queue: &Queue, namespace: Uuid, retailer: RetailerCode) {
        if self.acquire_token && self.token_event.is_none() {
            self.token_failure = queue.failure(
                stable_command_id(&namespace, AcquireStorefrontToken::NAME.as_bytes()),
                StorefrontTokenAcquired::NAME,
                self.initialization,
            );
            return;
        }
        let commands = self.catalog_commands(namespace, retailer);
        for (index, command) in commands.into_iter().enumerate() {
            if self.catalogs[index].failure.is_some() {
                continue;
            }
            if self.catalogs[index].event.is_none() {
                let trigger = if self.acquire_token {
                    self.token_event
                } else {
                    self.initialization
                };
                self.catalogs[index].failure =
                    queue.failure(command.id, ProductCatalogIdsRetrieved::NAME, trigger);
                continue;
            }
            let products: Vec<_> = self.catalogs[index].products.keys().cloned().collect();
            for id in products {
                let product = &self.catalogs[index].products[&id];
                if product.failure.is_some()
                    || product.skipped.is_some()
                    || product.structuralized.is_some()
                {
                    continue;
                }
                let (command, result, trigger) = if let Some(extracted) = &product.extracted {
                    (
                        stable_command_id(
                            &extracted.object,
                            StructuralizeOfferDetails::NAME.as_bytes(),
                        ),
                        OfferDetailsStructuralized::NAME,
                        Some(extracted.event),
                    )
                } else if let Some(fetched) = &product.fetched {
                    (
                        stable_command_id(&fetched.object, ExtractOfferDetails::NAME.as_bytes()),
                        OfferDetailsExtracted::NAME,
                        Some(fetched.event),
                    )
                } else {
                    (
                        self.detail_command(namespace, retailer, index, &id).id,
                        OfferDetailsFetched::NAME,
                        self.catalogs[index].event,
                    )
                };
                self.catalogs[index]
                    .products
                    .get_mut(&id)
                    .expect("known product")
                    .failure = queue.failure(command, result, trigger);
            }
        }
    }
}
