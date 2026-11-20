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
    KernelBuilder, KernelConfig, KernelError, NewEventEnvelope, NewEventMetadata,
    PersistentKernelHandle,
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

/// A decoded event together with the metadata carried by its stored envelope.
#[derive(Debug, Clone)]
pub struct EventEnvelope<E> {
    pub event: E,
    pub metadata: Option<NewEventMetadata>,
}

impl<E: Event> TryFrom<&NewEventEnvelope> for EventEnvelope<E> {
    type Error = EventError;

    fn try_from(envelope: &NewEventEnvelope) -> Result<Self, Self::Error> {
        if envelope.event_type != E::NAME {
            return Err(subscriber_error(format!(
                "expected event {}, received {}",
                E::NAME,
                envelope.event_type
            )));
        }
        Ok(Self {
            event: from_str(&envelope.payload).map_err(subscriber_error)?,
            metadata: envelope.metadata.clone(),
        })
    }
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

type BatchObserver = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

type Registration = Box<dyn FnOnce(KernelBuilder) -> KernelBuilder>;

/// Collects handlers and reactions until [`Bus::start`] runs them.
#[derive(Default)]
pub struct Bus {
    registrations: Vec<Registration>,
    batch_observers: Vec<BatchObserver>,
}

impl Bus {
    /// Observe durable outcomes after manual queue batches, outside the async runtime.
    pub fn after_batch(
        &mut self,
        observer: impl Fn() -> Result<(), String> + Send + Sync + 'static,
    ) {
        self.batch_observers.push(Arc::new(observer));
    }

    /// Registers the handler for its command type.
    pub fn handle<H: Handle>(&mut self, handler: H) {
        let port: Arc<dyn CommandHandlerPort<H::Command, Published<H::Event>>> =
            Arc::new(HandlerPort(handler));
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

    /// A replay-safe reaction: each command supplies a stable identity and inherits event correlation.
    #[cfg(test)]
    pub fn on_many_keyed<E: Event, C: Command>(
        &mut self,
        reaction: fn(&E) -> Vec<KeyedCommand<C>>,
    ) {
        self.registrations.push(Box::new(move |kernel| {
            kernel.event_subscriber_with_command_gateway(E::NAME, C::NAME, move |gateway| {
                Arc::new(KeyedReaction { gateway, reaction }) as Arc<dyn EventSubscriberPort>
            })
        }));
    }

    /// Registers a command-producing reaction with the decoded event and its metadata.
    pub fn on_commands_envelope<E: Event, C: IntoEventCommand, Err: Display + 'static>(
        &mut self,
        name: &'static str,
        reaction: impl Fn(&EventEnvelope<E>) -> Result<Vec<C>, Err> + Send + Sync + 'static,
    ) {
        let reaction: CommandReactionFn<E, C, Err> = Arc::new(reaction);
        self.registrations.push(Box::new(move |kernel| {
            kernel.event_subscriber_with_command_gateway(E::NAME, name, move |gateway| {
                Arc::new(CommandReaction {
                    gateway,
                    reaction: reaction.clone(),
                })
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
            #[cfg(test)]
            dispatcher: Dispatcher {
                gateway: handle.state().command_gateway(),
            },
            commands: CommandingAdapter::new(AsyncCommandingService::new(
                handle.command_consumer(),
            )),
            events: EventingAdapter::new(AsyncEventingService::new(handle.event_consumer())),
            kernel: Arc::new(handle),
            batch_observers: self.batch_observers,
        })
    }
}

/// A started bus: queues commands and processes queued work on request.
#[derive(Clone)]
pub struct BusHandle {
    #[cfg(test)]
    dispatcher: Dispatcher,
    pub(super) commands: CommandingAdapter,
    pub(super) events: EventingAdapter,
    kernel: Arc<PersistentKernelHandle>,
    batch_observers: Vec<BatchObserver>,
}

impl BusHandle {
    pub(super) fn observe_batch(&self) -> Result<(), String> {
        for observer in &self.batch_observers {
            observer()?;
        }
        Ok(())
    }

    pub fn command_gateway(&self) -> Arc<CommandGateway> {
        self.kernel.state().command_gateway()
    }

    #[cfg(test)]
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
#[cfg(test)]
#[derive(Clone)]
pub struct Dispatcher {
    gateway: Arc<CommandGateway>,
}

/// A command that couldn't be queued.
#[cfg(test)]
#[derive(Debug, thiserror::Error)]
#[error("failed to queue command: {0}")]
pub struct DispatchError(String);

#[cfg(test)]
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

/// A command whose identity survives replay of its triggering event.
#[derive(Debug)]
pub struct KeyedCommand<C> {
    pub id: Uuid,
    pub command: C,
}

pub trait IntoEventCommand: Send + Sync + 'static {
    fn encode(self) -> Result<KeyedCommand<NewCommand>, SerdeJsonError>;
}

impl<C: Command> KeyedCommand<C> {
    pub fn encode(self) -> Result<KeyedCommand<NewCommand>, SerdeJsonError> {
        Ok(KeyedCommand {
            id: self.id,
            command: NewCommand {
                command_type: C::NAME.into(),
                payload: serde_json_to_string(&self.command)?,
            },
        })
    }
}

type CommandReactionFn<E, C, Err> =
    Arc<dyn Fn(&EventEnvelope<E>) -> Result<Vec<C>, Err> + Send + Sync>;

struct CommandReaction<E, C, Err> {
    gateway: Arc<CommandGateway>,
    reaction: CommandReactionFn<E, C, Err>,
}

impl<E: Event, C: IntoEventCommand, Err: Display> EventSubscriberPort
    for CommandReaction<E, C, Err>
{
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        let event = EventEnvelope::<E>::try_from(envelope)?;
        for command in (self.reaction)(&event).map_err(subscriber_error)? {
            let metadata = event
                .metadata
                .as_ref()
                .ok_or_else(|| subscriber_error("command reaction requires event metadata"))?;
            let keyed = command.encode().map_err(subscriber_error)?;
            self.gateway
                .dispatch(NewCommandEnvelope {
                    command: keyed.command,
                    metadata: Some(NewCommandMetadata {
                        command_id: keyed.id,
                        correlation_id: metadata.correlation_id,
                        causation_id: Some(metadata.event_id),
                        source: Some(format!("event:{}", E::NAME)),
                    }),
                })
                .map_err(subscriber_error)?;
        }
        Ok(())
    }
}

#[cfg(test)]
struct KeyedReaction<E, C> {
    gateway: Arc<CommandGateway>,
    reaction: fn(&E) -> Vec<KeyedCommand<C>>,
}

#[cfg(test)]
impl<E: Event, C: Command> EventSubscriberPort for KeyedReaction<E, C> {
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        let event: E = from_str(&envelope.payload).map_err(subscriber_error)?;
        let metadata = envelope
            .metadata
            .as_ref()
            .ok_or_else(|| subscriber_error("keyed reaction requires event metadata"))?;
        let start = Instant::now();
        let result = (|| {
            for keyed in (self.reaction)(&event) {
                let command = command_envelope(
                    &keyed.command,
                    NewCommandMetadata {
                        command_id: keyed.id,
                        correlation_id: metadata.correlation_id,
                        causation_id: Some(metadata.event_id),
                        source: Some(format!("event:{}", E::NAME)),
                    },
                )
                .map_err(subscriber_error)?;
                self.gateway.dispatch(command).map_err(subscriber_error)?;
            }
            Ok(())
        })();
        info!(target: TIMING_TARGET, kind="event", name=E::NAME, reaction=C::NAME, elapsed_ms=start.elapsed().as_millis() as u64, status=if result.is_ok() { "ok" } else { "error" });
        result
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

struct HandlerPort<H>(H);

impl<H: Handle> CommandHandlerPort<H::Command, Published<H::Event>> for HandlerPort<H> {
    fn execute(&self, command: H::Command) -> Result<Vec<Published<H::Event>>, CommandError> {
        let name = <H::Command as Command>::NAME;
        let start = Instant::now();
        let result = self.0.handle(command);
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
        let name = <L::Event as Event>::NAME;
        let start = Instant::now();
        let result = self.listen(envelope);
        let elapsed_ms = start.elapsed().as_millis() as u64;
        let status = if result.is_ok() { "ok" } else { "error" };
        info!(
            target: TIMING_TARGET,
            kind = "event",
            name,
            listener = L::NAME,
            elapsed_ms,
            status
        );
        if let Err(error) = &result {
            tracing_error!(event = name, listener = L::NAME, "listener failed: {error}");
        }
        result
    }
}

impl<L: Listen> Listener<L> {
    fn listen(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        let event: L::Event = from_str(&envelope.payload).map_err(subscriber_error)?;
        let metadata = envelope.metadata.as_ref().ok_or_else(|| {
            subscriber_error(format!(
                "event {} has no envelope metadata",
                <L::Event as Event>::NAME
            ))
        })?;
        let context = EventContext {
            event_id: metadata.event_id,
            correlation_id: metadata.correlation_id,
            causation_id: metadata.causation_id,
            source: metadata.source.clone(),
        };
        self.0.listen(event, &context).map_err(subscriber_error)
    }
}

pub(crate) fn command_envelope<C: Command>(
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
    fn typed_envelopes_preserve_payload_and_all_metadata() {
        let mut metadata = metadata();
        metadata.sequence = Some(7);
        let raw = ponged(42, Some(metadata.clone()));
        let envelope = EventEnvelope::<Ponged>::try_from(&raw).unwrap();
        assert_eq!(envelope.event.id, 42);
        let actual = envelope.metadata.unwrap();
        assert_eq!(actual.sequence, metadata.sequence);
        assert_eq!(actual.event_id, metadata.event_id);
        assert_eq!(actual.correlation_id, metadata.correlation_id);
        assert_eq!(actual.causation_id, metadata.causation_id);
        assert_eq!(actual.source, metadata.source);
        assert!(
            EventEnvelope::<Ponged>::try_from(&ponged(42, None))
                .unwrap()
                .metadata
                .is_none()
        );

        let mut wrong_type = raw.clone();
        wrong_type.event_type = "OtherEvent".into();
        assert!(EventEnvelope::<Ponged>::try_from(&wrong_type).is_err());
        let mut malformed = raw;
        malformed.payload = "invalid JSON".into();
        assert!(EventEnvelope::<Ponged>::try_from(&malformed).is_err());
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
        let events = HandlerPort(Echo).execute(Ping { id: 7 }).unwrap();

        assert_eq!(events[0].event_type(), "Ponged");
        assert_eq!(serde_json_to_string2(&events[0]).unwrap(), r#"{"id":7}"#);
    }

    #[test]
    fn reports_handler_failures_as_storage_errors() {
        let error = HandlerPort(Echo).execute(Ping { id: 0 }).err().unwrap();

        assert!(
            matches!(&error, CommandError::Storage(message) if message == "id must not be zero"),
            "{error:?}"
        );
    }
}

/// RFC 9562 UUIDv5 for replay-stable command identities. Uses the existing
/// RustCrypto SHA-1 implementation; these IDs are identifiers, not signatures.
pub fn stable_command_id(namespace: &Uuid, key: &[u8]) -> Uuid {
    use uuid::Builder;

    use sha1::{Digest, Sha1};
    let digest = Sha1::new()
        .chain_update(namespace.as_bytes())
        .chain_update(key)
        .finalize();
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    Builder::from_sha1_bytes(bytes).into_uuid()
}

#[cfg(test)]
#[path = "keyed_reaction_tests.rs"]
mod keyed_reaction_tests;
