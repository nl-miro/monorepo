//! Command and event bus on top of the Mulac kernel.
//!
//! Features register handlers, reactions and listeners here and never touch Mulac
//! directly. Mulac stores every command and event in Postgres, delivers each one
//! at least once, and retries a failed command with backoff.
//!
//! The executable explicitly starts command workers. The HTTP routes also
//! delegate bounded batches to Mulac's command and event adapters.

use kernel::boot;
use kernel::prelude::run_command_worker;
use serde_json::Error as SerdeJsonError;
use serde_json::from_str;
use serde_json::to_string as serde_json_to_string;
use tokio::spawn;
use tokio::task::JoinError;
use tokio::task::JoinHandle;
use tracing::error as tracing_error;
use tracing::info;

use std::fmt::Display;
use std::sync::Arc;
use std::time::Instant;

use commanding::prelude::AsyncCommandingService;
use commanding::prelude::axum::CommandingAdapter;
use eventing::prelude::AsyncEventingService;
use eventing::prelude::axum::EventingAdapter;
use kernel::prelude::{CommandGateway, NewCommand, NewCommandEnvelope, NewCommandMetadata};
use kernel::{
    ApplicationEvent, CommandError, CommandHandlerPort, EventError, EventSubscriberPort,
    KernelBuilder, KernelConfig, KernelError, NewEventEnvelope, PersistentKernelHandle,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use super::db::DbPool;
use super::logging::TIMING_TARGET;

/// A request to change state, handled by exactly one handler.
pub trait Command: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// The stored command type. Changing it orphans queued commands.
    const NAME: &'static str;
}

/// A fact that something happened, which any number of reactions may follow.
pub trait Event: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// The stored event type. Changing it orphans queued events.
    const NAME: &'static str;
}

/// What a handler returns for the bus to store as an event: any [`Event`], or
/// an enum of events when a handler can produce more than one kind. The enum
/// must serialize as its variant's event alone (`#[serde(untagged)]`), so
/// reactions read it as that event.
pub trait Publish: Serialize + Send + Sync + 'static {
    fn event_type(&self) -> &'static str;
}

impl<E: Event> Publish for E {
    fn event_type(&self) -> &'static str {
        E::NAME
    }
}

/// Where an event came from, as recorded in its envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventContext {
    pub event_id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub source: Option<String>,
}

/// Consumes one event type with its envelope metadata, for subscribers that
/// write state themselves rather than queue a command.
pub trait Listen: Send + Sync + 'static {
    type Event: Event;
    type Error: Display;

    /// The subscriber name Mulac registers.
    const NAME: &'static str;

    fn listen(&self, event: Self::Event, context: &EventContext) -> Result<(), Self::Error>;
}

/// Handles one command type and returns the events it produced. Handlers
/// don't publish events themselves.
pub trait Handle: Send + Sync + 'static {
    type Command: Command;
    type Event: Publish;
    type Error: Display;

    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}

type Registration = Box<dyn FnOnce(KernelBuilder) -> KernelBuilder>;

/// Collects handlers and reactions until [`Bus::start`] runs them.
#[derive(Default)]
pub struct Bus {
    registrations: Vec<Registration>,
}

impl Bus {
    /// Registers the handler for its command type.
    pub fn handle<H: Handle>(&mut self, handler: H) {
        let port: Arc<dyn CommandHandlerPort<H::Command, Published<H::Event>>> =
            Arc::new(HandlerPort { handler });
        self.registrations.push(Box::new(move |kernel| {
            kernel.command_handler(<H::Command as Command>::NAME, port)
        }));
    }

    /// Registers a reaction that turns each `E` event into a `C` command.
    pub fn on<E, C>(&mut self, reaction: fn(&E) -> Option<C>)
    where
        E: Event,
        C: Command,
    {
        self.registrations.push(Box::new(move |kernel| {
            kernel.event_subscriber_with_command_gateway(E::NAME, C::NAME, move |gateway| {
                Arc::new(Reaction { gateway, reaction }) as Arc<dyn EventSubscriberPort>
            })
        }));
    }

    /// Registers a listener for its event type.
    pub fn listen<L: Listen>(&mut self, listener: L) {
        let port: Arc<dyn EventSubscriberPort> = Arc::new(Listener(listener));
        self.registrations.push(Box::new(move |kernel| {
            kernel.event_subscriber(<L::Event as Event>::NAME, L::NAME, port)
        }));
    }

    /// Registers one typed method on a shared listener. Register other methods
    /// on the same `Arc` to handle additional event types. Keep subscription
    /// names stable; sharing a listener does not serialize event processing.
    pub fn listen_method<L, E, Err>(
        &mut self,
        name: &'static str,
        listener: Arc<L>,
        method: fn(&L, E, &EventContext) -> Result<(), Err>,
    ) where
        L: Send + Sync + 'static,
        E: Event,
        Err: Display + 'static,
    {
        let port: Arc<dyn EventSubscriberPort> = Arc::new(MethodListener {
            name,
            listener,
            method,
        });
        self.registrations.push(Box::new(move |kernel| {
            kernel.event_subscriber(E::NAME, name, port)
        }));
    }

    /// Registers a method whose successful result may enqueue a batch of commands.
    pub(crate) fn listen_command_method<L, E, C, Err>(
        &mut self,
        name: &'static str,
        listener: Arc<L>,
        method: CommandMethod<L, E, C, Err>,
    ) where
        L: Send + Sync + 'static,
        E: Event,
        C: EventCommand + 'static,
        Err: Display + 'static,
    {
        self.registrations.push(Box::new(move |kernel| {
            kernel.event_subscriber_with_command_gateway(E::NAME, name, move |gateway| {
                Arc::new(CommandMethodListener {
                    name,
                    listener: Arc::clone(&listener),
                    method,
                    gateway,
                    standalone: None,
                })
            })
        }));
    }

    /// Registers a command method on an event also produced by standalone work.
    /// Keep the original reaction's subscription name and metadata on that path.
    pub(crate) fn listen_command_method_or_react<L, E, C, R, Err>(
        &mut self,
        reaction: fn(&E) -> Option<R>,
        listener: Arc<L>,
        method: CommandMethod<L, E, C, Err>,
        is_standalone: fn(&Err) -> bool,
    ) where
        L: Send + Sync + 'static,
        E: Event,
        C: EventCommand + 'static,
        R: Command,
        Err: Display + 'static,
    {
        self.registrations.push(Box::new(move |kernel| {
            kernel.event_subscriber_with_command_gateway(E::NAME, R::NAME, move |gateway| {
                let standalone = StandaloneReaction {
                    is_standalone,
                    subscriber: Arc::new(Reaction {
                        gateway: Arc::clone(&gateway),
                        reaction,
                    }),
                };
                Arc::new(CommandMethodListener {
                    name: R::NAME,
                    listener: Arc::clone(&listener),
                    method,
                    gateway,
                    standalone: Some(standalone),
                }) as Arc<dyn EventSubscriberPort>
            })
        }));
    }

    /// Builds the bus over `pool`. Queued work runs only through the returned
    /// handle.
    pub fn start(self, pool: DbPool) -> Result<BusHandle, KernelError> {
        let kernel = self
            .registrations
            .into_iter()
            .fold(boot(KernelConfig::default()), |kernel, register| {
                register(kernel)
            });
        let handle = kernel.start_persistent(pool, 0)?;

        Ok(BusHandle {
            dispatcher: Dispatcher {
                gateway: handle.state().command_gateway(),
            },
            commands: CommandingAdapter::new(AsyncCommandingService::new(
                handle.command_consumer(),
            )),
            events: EventingAdapter::new(AsyncEventingService::new(handle.event_consumer())),
            kernel: Arc::new(handle),
        })
    }
}

/// A started bus: queues commands and processes queued work on request.
#[derive(Clone)]
pub struct BusHandle {
    dispatcher: Dispatcher,
    pub(super) commands: CommandingAdapter,
    pub(super) events: EventingAdapter,
    kernel: Arc<PersistentKernelHandle>,
}

impl BusHandle {
    pub fn dispatcher(&self) -> &Dispatcher {
        &self.dispatcher
    }

    /// Starts exactly `count` command loops; events remain manual. Bus construction
    /// itself stays worker-free so callers control the workers' lifetime.
    pub fn start_command_workers(&self, count: usize) -> CommandWorkers {
        let handles = (0..count)
            .map(|_| {
                spawn(run_command_worker(
                    self.kernel.command_consumer(),
                    self.kernel.child_token(),
                ))
            })
            .collect();
        if count == 0 {
            info!(
                command_workers = count,
                "command workers disabled: manual command processing"
            );
        } else {
            info!(command_workers = count, "command workers started");
        }
        CommandWorkers { handles }
    }

    /// Cancel workers immediately; an already reserved batch finishes before
    /// its worker exits. The caller must also join the workers.
    pub fn shutdown(&self) {
        self.kernel.shutdown();
    }
}

/// All command loops owned by the executable, joined after HTTP drains.
#[derive(Default)]
pub struct CommandWorkers {
    handles: Vec<JoinHandle<()>>,
}

impl CommandWorkers {
    pub async fn join(self) -> Result<(), JoinError> {
        let count = self.handles.len();
        let mut first_error = None;
        for handle in self.handles {
            if let Err(error) = handle.await {
                tracing_error!(%error, "command worker failed to join");
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
        info!(command_workers = count, "command workers stopped");
        first_error.map_or(Ok(()), Err)
    }
}

#[cfg(test)]
#[path = "command_workers_tests.rs"]
mod command_workers_tests;

/// Queues commands from outside the bus, such as from an HTTP request.
#[derive(Clone)]
pub struct Dispatcher {
    gateway: Arc<CommandGateway>,
}

/// A command that couldn't be queued.
#[derive(Debug, thiserror::Error)]
#[error("failed to queue command: {0}")]
pub struct DispatchError(String);

impl Dispatcher {
    /// Stores `command` for the workers to handle. `source` says where it came
    /// from, such as `api`. Blocks on a database insert, so call it off the
    /// async runtime.
    pub fn dispatch<C: Command>(&self, command: &C, source: &str) -> Result<(), DispatchError> {
        let envelope = command_envelope(
            command,
            NewCommandMetadata {
                command_id: Uuid::now_v7(),
                correlation_id: None,
                causation_id: None,
                source: Some(source.to_string()),
            },
        )
        .map_err(|error| DispatchError(error.to_string()))?;

        self.gateway
            .dispatch(envelope)
            .map_err(|error| DispatchError(error.to_string()))
    }
}

/// Serializes an event exactly as the feature defined it, tagged with its name.
#[derive(Serialize)]
#[serde(transparent)]
struct Published<E>(E);

impl<E: Publish> ApplicationEvent for Published<E> {
    fn event_type(&self) -> &'static str {
        self.0.event_type()
    }
}

struct HandlerPort<H> {
    handler: H,
}

impl<H: Handle> CommandHandlerPort<H::Command, Published<H::Event>> for HandlerPort<H> {
    fn execute(&self, command: H::Command) -> Result<Vec<Published<H::Event>>, CommandError> {
        let name = <H::Command as Command>::NAME;
        let start = Instant::now();
        let result = self.handler.handle(command);
        let elapsed_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(events) => {
                info!(target: TIMING_TARGET, kind = "command", name, elapsed_ms, status = "ok");
                Ok(events.into_iter().map(Published).collect())
            }
            Err(error) => {
                info!(target: TIMING_TARGET, kind = "command", name, elapsed_ms, status = "error");
                tracing_error!(command = name, "command failed: {error}");
                // Every failure is reported as a storage error, so Mulac
                // retries all of them. Splitting retryable from permanent
                // errors is a follow-up.
                Err(CommandError::Storage(error.to_string()))
            }
        }
    }
}

struct Reaction<E, C> {
    gateway: Arc<CommandGateway>,
    reaction: fn(&E) -> Option<C>,
}

impl<E, C> EventSubscriberPort for Reaction<E, C>
where
    E: Event,
    C: Command,
{
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        let start = Instant::now();
        let result = self.react(envelope);
        let elapsed_ms = start.elapsed().as_millis() as u64;
        let status = if result.is_ok() { "ok" } else { "error" };
        info!(
            target: TIMING_TARGET,
            kind = "event",
            name = E::NAME,
            reaction = C::NAME,
            elapsed_ms,
            status
        );
        if let Err(error) = &result {
            tracing_error!(
                event = E::NAME,
                reaction = C::NAME,
                "reaction failed: {error}"
            );
        }
        result
    }
}

impl<E, C> Reaction<E, C>
where
    E: Event,
    C: Command,
{
    fn react(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        let event: E = from_str(&envelope.payload).map_err(subscriber_error)?;
        let Some(command) = (self.reaction)(&event) else {
            return Ok(());
        };

        let metadata = envelope.metadata.as_ref();
        let command = command_envelope(
            &command,
            NewCommandMetadata {
                command_id: Uuid::now_v7(),
                correlation_id: metadata.and_then(|metadata| metadata.correlation_id),
                causation_id: metadata.map(|metadata| metadata.event_id),
                source: Some(format!("event:{}", E::NAME)),
            },
        )
        .map_err(subscriber_error)?;

        self.gateway.dispatch(command).map_err(subscriber_error)
    }
}

struct Listener<L>(L);

impl<L: Listen> EventSubscriberPort for Listener<L> {
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        dispatch_listener(L::NAME, envelope, |event, context| {
            self.0.listen(event, context)
        })
    }
}

struct MethodListener<L, E, Err> {
    name: &'static str,
    listener: Arc<L>,
    method: fn(&L, E, &EventContext) -> Result<(), Err>,
}

impl<L, E, Err> EventSubscriberPort for MethodListener<L, E, Err>
where
    L: Send + Sync + 'static,
    E: Event,
    Err: Display + 'static,
{
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        dispatch_listener(self.name, envelope, |event, context| {
            (self.method)(&self.listener, event, context)
        })
    }
}

pub(crate) trait EventCommand {
    fn into_envelope(
        self,
        correlation_id: Uuid,
        event_name: &str,
        context: &EventContext,
    ) -> Result<NewCommandEnvelope, SerdeJsonError>;
}

pub(crate) fn event_command_envelope<C: Command>(
    command: &C,
    command_id: Uuid,
    correlation_id: Uuid,
    event_name: &str,
    context: &EventContext,
) -> Result<NewCommandEnvelope, SerdeJsonError> {
    command_envelope(
        command,
        NewCommandMetadata {
            command_id,
            correlation_id: Some(correlation_id),
            causation_id: Some(context.event_id),
            source: Some(format!("event:{event_name}")),
        },
    )
}

type CommandMethod<L, E, C, Err> = fn(&L, E, &EventContext) -> Result<(Uuid, Vec<C>), Err>;

struct StandaloneReaction<Err> {
    is_standalone: fn(&Err) -> bool,
    subscriber: Arc<dyn EventSubscriberPort>,
}

struct CommandMethodListener<L, E, C, Err> {
    standalone: Option<StandaloneReaction<Err>>,
    name: &'static str,
    listener: Arc<L>,
    method: CommandMethod<L, E, C, Err>,
    gateway: Arc<CommandGateway>,
}

impl<L: Send + Sync, E: Event, C: EventCommand, Err: Display> EventSubscriberPort
    for CommandMethodListener<L, E, C, Err>
{
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        if envelope.metadata.is_none()
            && let Some(standalone) = &self.standalone
        {
            return standalone.subscriber.handle(envelope);
        }
        dispatch_listener(self.name, envelope, |event, context| {
            let (correlation_id, commands) = match (self.method)(&self.listener, event, context) {
                Ok(batch) => batch,
                Err(error) => {
                    if let Some(standalone) = &self.standalone
                        && (standalone.is_standalone)(&error)
                    {
                        return standalone
                            .subscriber
                            .handle(envelope)
                            .map_err(|e| e.to_string());
                    }
                    return Err(error.to_string());
                }
            };
            for command in commands {
                let envelope = command
                    .into_envelope(correlation_id, E::NAME, context)
                    .map_err(|e| e.to_string())?;
                self.gateway.dispatch(envelope).map_err(|e| e.to_string())?;
            }
            Ok::<(), String>(())
        })
    }
}

fn dispatch_listener<E: Event, Err: Display>(
    listener: &str,
    envelope: &NewEventEnvelope,
    method: impl FnOnce(E, &EventContext) -> Result<(), Err>,
) -> Result<(), EventError> {
    let name = E::NAME;
    let start = Instant::now();
    let result = (|| {
        let event: E = from_str(&envelope.payload).map_err(subscriber_error)?;
        let metadata = envelope
            .metadata
            .as_ref()
            .ok_or_else(|| subscriber_error(format!("event {name} has no envelope metadata")))?;
        let context = EventContext {
            event_id: metadata.event_id,
            correlation_id: metadata.correlation_id,
            causation_id: metadata.causation_id,
            source: metadata.source.clone(),
        };
        method(event, &context).map_err(subscriber_error)
    })();
    let elapsed_ms = start.elapsed().as_millis() as u64;
    let status = if result.is_ok() { "ok" } else { "error" };
    info!(
        target: TIMING_TARGET,
        kind = "event",
        name,
        listener,
        elapsed_ms,
        status
    );
    if let Err(error) = &result {
        tracing_error!(event = name, listener, "listener failed: {error}");
    }
    result
}

fn command_envelope<C: Command>(
    command: &C,
    metadata: NewCommandMetadata,
) -> Result<NewCommandEnvelope, SerdeJsonError> {
    Ok(NewCommandEnvelope {
        command: NewCommand {
            command_type: C::NAME.to_string(),
            payload: serde_json_to_string(command)?,
        },
        metadata: Some(metadata),
    })
}

fn subscriber_error(error: impl Display) -> EventError {
    EventError::SubscriberExecution(error.to_string())
}

#[cfg(test)]
mod tests {
    use kernel::NewEventMetadata;
    use serde_json::to_string as serde_json_to_string2;
    use std::sync::Mutex;

    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Ping {
        id: u32,
    }

    impl Command for Ping {
        const NAME: &'static str = "Ping";
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Ponged {
        id: u32,
    }

    impl Event for Ponged {
        const NAME: &'static str = "Ponged";
    }

    struct Echo;

    impl Handle for Echo {
        type Command = Ping;
        type Event = Ponged;
        type Error = String;

        fn handle(&self, command: Ping) -> Result<Vec<Ponged>, String> {
            if command.id == 0 {
                return Err("id must not be zero".to_string());
            }
            Ok(vec![Ponged { id: command.id }])
        }
    }

    struct Record(Mutex<Vec<(u32, EventContext)>>);

    impl Listen for Record {
        type Event = Ponged;
        type Error = String;
        const NAME: &'static str = "RecordPonged";

        fn listen(&self, event: Ponged, context: &EventContext) -> Result<(), String> {
            if event.id == 0 {
                return Err("id must not be zero".to_string());
            }
            self.0.lock().unwrap().push((event.id, context.clone()));
            Ok(())
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Finished {
        total: u32,
    }

    impl Event for Finished {
        const NAME: &'static str = "Finished";
    }

    impl Record {
        fn on_ponged(&self, event: Ponged, context: &EventContext) -> Result<(), String> {
            self.listen(event, context)
        }

        fn on_finished(&self, event: Finished, context: &EventContext) -> Result<(), String> {
            self.0.lock().unwrap().push((event.total, context.clone()));
            Ok(())
        }
    }

    #[test]
    fn methods_on_one_listener_handle_different_event_types() {
        let record = Arc::new(Record(Default::default()));
        let pong_listener = MethodListener {
            name: "RecordPonged",
            listener: Arc::clone(&record),
            method: Record::on_ponged,
        };
        let finished_listener = MethodListener {
            name: "RecordFinished",
            listener: Arc::clone(&record),
            method: Record::on_finished,
        };
        let metadata = metadata();
        pong_listener
            .handle(&ponged(7, Some(metadata.clone())))
            .unwrap();
        finished_listener
            .handle(&NewEventEnvelope {
                event_type: Finished::NAME.into(),
                payload: serde_json_to_string2(&Finished { total: 42 }).unwrap(),
                metadata: Some(metadata.clone()),
            })
            .unwrap();

        let calls = record.0.lock().unwrap();
        assert_eq!(
            calls.iter().map(|(value, _)| *value).collect::<Vec<_>>(),
            [7, 42]
        );
        for (_, context) in calls.iter() {
            assert_eq!(context.event_id, metadata.event_id);
            assert_eq!(context.correlation_id, metadata.correlation_id);
            assert_eq!(context.causation_id, metadata.causation_id);
            assert_eq!(context.source, metadata.source);
        }
    }

    #[test]
    fn method_listeners_report_errors_without_recording_invalid_events() {
        let record = Arc::new(Record(Default::default()));
        let listener = MethodListener {
            name: "RecordPonged",
            listener: Arc::clone(&record),
            method: Record::on_ponged,
        };
        let missing = listener.handle(&ponged(7, None)).unwrap_err();
        assert!(matches!(missing, EventError::SubscriberExecution(message)
            if message.contains("no envelope metadata")));
        let failed = listener.handle(&ponged(0, Some(metadata()))).unwrap_err();
        assert!(matches!(failed, EventError::SubscriberExecution(message)
            if message == "id must not be zero"));
        let mut malformed = ponged(7, Some(metadata()));
        malformed.payload = "not json".into();
        assert!(matches!(
            listener.handle(&malformed),
            Err(EventError::SubscriberExecution(_))
        ));
        assert!(record.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn method_registrations_preserve_event_types_and_subscription_names() {
        let record = Arc::new(Record(Default::default()));
        let mut bus = Bus::default();
        bus.listen_method("Record", Arc::clone(&record), Record::on_ponged);
        bus.listen_method("Record", Arc::clone(&record), Record::on_finished);
        let kernel = bus
            .registrations
            .into_iter()
            .fold(boot(KernelConfig::default()), |kernel, register| {
                register(kernel)
            });
        // Both subscriptions validate; only the deliberately omitted inbox store is missing.
        assert!(matches!(
            kernel.start().await,
            Err(KernelError::InboxStoreRequired)
        ));

        let mut bus = Bus::default();
        bus.listen_method("Record", Arc::clone(&record), Record::on_ponged);
        bus.listen_method("Record", record, Record::on_ponged);
        let kernel = bus
            .registrations
            .into_iter()
            .fold(boot(KernelConfig::default()), |kernel, register| {
                register(kernel)
            });
        assert!(matches!(kernel.start().await,
            Err(KernelError::DuplicateEventSubscriber { event_type, subscriber })
                if event_type == "Ponged" && subscriber == "Record"));
    }

    mod postgres_tests {
        use super::*;
        use crate::platform::test_db::Database;
        use diesel::RunQueryDsl;
        use diesel::sql_query;
        use diesel::sql_types::{Jsonb, Text, Uuid as SqlUuid};
        use kernel::prelude::ReservableEventSpec;
        use serde_json::to_value;

        #[tokio::test]
        #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
        async fn routes_persisted_events_to_methods_on_the_same_listener() {
            let db = Database::new();
            let record = Arc::new(Record(Default::default()));
            let mut bus = Bus::default();
            bus.listen_method("RecordPonged", Arc::clone(&record), Record::on_ponged);
            bus.listen_method("RecordFinished", Arc::clone(&record), Record::on_finished);
            let bus = bus.start(db.pool.clone()).unwrap();
            let first_metadata = metadata();
            let second_metadata = metadata();
            let envelopes = [
                ponged(7, Some(first_metadata.clone())),
                NewEventEnvelope {
                    event_type: Finished::NAME.into(),
                    payload: serde_json_to_string2(&Finished { total: 42 }).unwrap(),
                    metadata: Some(second_metadata.clone()),
                },
            ];
            for envelope in envelopes {
                let metadata = envelope.metadata.unwrap();
                sql_query(
                    "INSERT INTO event_entries \
                     (id, event_type, payload, meta, status, attempts, scheduled_at, received_at, updated_at, extra_info) \
                     VALUES ($1, $2, $3, $4, 4, 0, now() - interval '1 second', now(), now(), '{}')"
                )
                .bind::<SqlUuid, _>(metadata.event_id)
                .bind::<Text, _>(envelope.event_type)
                .bind::<Text, _>(envelope.payload)
                .bind::<Jsonb, _>(to_value(metadata).unwrap())
                .execute(&mut db.pool.get().unwrap())
                .unwrap();
            }
            assert!(record.0.lock().unwrap().is_empty());
            assert_eq!(
                bus.kernel
                    .event_consumer()
                    .consume(&ReservableEventSpec::new(10))
                    .unwrap(),
                2
            );
            let mut calls = record.0.lock().unwrap().clone();
            calls.sort_by_key(|(value, _)| *value);
            assert_eq!(calls.len(), 2);
            for ((value, context), (expected, metadata)) in calls
                .iter()
                .zip([(7, first_metadata), (42, second_metadata)])
            {
                assert_eq!(*value, expected);
                assert_eq!(
                    *context,
                    EventContext {
                        event_id: metadata.event_id,
                        correlation_id: metadata.correlation_id,
                        causation_id: metadata.causation_id,
                        source: metadata.source,
                    }
                );
            }
            assert_eq!(
                bus.kernel
                    .event_consumer()
                    .consume(&ReservableEventSpec::new(10))
                    .unwrap(),
                0
            );
            bus.shutdown();
        }
    }

    fn ponged(id: u32, metadata: Option<NewEventMetadata>) -> NewEventEnvelope {
        NewEventEnvelope {
            event_type: Ponged::NAME.to_string(),
            payload: serde_json_to_string2(&Ponged { id }).unwrap(),
            metadata,
        }
    }

    fn metadata() -> NewEventMetadata {
        NewEventMetadata {
            sequence: None,
            event_id: Uuid::now_v7(),
            correlation_id: Some(Uuid::now_v7()),
            causation_id: Some(Uuid::now_v7()),
            source: Some("test".to_string()),
        }
    }

    #[test]
    fn listeners_receive_the_envelope_metadata() {
        let listener = Listener(Record(Default::default()));
        let metadata = metadata();

        listener.handle(&ponged(7, Some(metadata.clone()))).unwrap();

        let calls = listener.0.0.lock().unwrap();
        assert_eq!(
            *calls,
            [(
                7,
                EventContext {
                    event_id: metadata.event_id,
                    correlation_id: metadata.correlation_id,
                    causation_id: metadata.causation_id,
                    source: metadata.source,
                }
            )]
        );
    }

    #[test]
    fn listeners_reject_events_without_metadata_and_report_failures() {
        let listener = Listener(Record(Default::default()));

        let missing = listener.handle(&ponged(7, None)).unwrap_err();
        let failed = listener.handle(&ponged(0, Some(metadata()))).unwrap_err();

        assert!(
            matches!(&missing, EventError::SubscriberExecution(message) if message.contains("no envelope metadata")),
            "{missing:?}"
        );
        assert!(
            matches!(&failed, EventError::SubscriberExecution(message) if message == "id must not be zero"),
            "{failed:?}"
        );
        assert!(listener.0.0.lock().unwrap().is_empty());
    }

    #[test]
    fn publishes_events_as_the_feature_serializes_them() {
        let events = HandlerPort { handler: Echo }
            .execute(Ping { id: 7 })
            .unwrap();

        assert_eq!(events[0].event_type(), "Ponged");
        assert_eq!(serde_json_to_string2(&events[0]).unwrap(), r#"{"id":7}"#);
    }

    #[test]
    fn reports_handler_failures_as_storage_errors() {
        let error = HandlerPort { handler: Echo }
            .execute(Ping { id: 0 })
            .err()
            .unwrap();

        assert!(
            matches!(&error, CommandError::Storage(message) if message == "id must not be zero"),
            "{error:?}"
        );
    }
}
