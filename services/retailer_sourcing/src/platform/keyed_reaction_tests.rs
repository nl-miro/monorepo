use kernel::NewEventMetadata;
use serde_json::from_str;
use serde_json::to_string as serde_json_to_string;

use super::*;
use commanding::prelude::CommandStorePort;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Serialize, Deserialize)]
struct Batch {
    run_id: Uuid,
    ids: Vec<u32>,
}
impl Event for Batch {
    const NAME: &'static str = "TestBatch";
}
#[derive(Serialize, Deserialize)]
struct Work {
    id: u32,
}
impl Command for Work {
    const NAME: &'static str = "TestBatchWork";
}
impl IntoEventCommand for KeyedCommand<Work> {
    fn encode(self) -> Result<KeyedCommand<NewCommand>, SerdeJsonError> {
        KeyedCommand::encode(self)
    }
}
fn commands(batch: &Batch) -> Vec<KeyedCommand<Work>> {
    batch
        .ids
        .iter()
        .map(|id| KeyedCommand {
            id: stable_command_id(&batch.run_id, &id.to_be_bytes()),
            command: Work { id: *id },
        })
        .collect()
}
#[derive(Default)]
struct Store {
    rows: Mutex<HashMap<Uuid, NewCommandEnvelope>>,
    failed: AtomicBool,
}
impl CommandStorePort for Store {
    fn record(&self, envelope: &NewCommandEnvelope) -> Result<(), CommandError> {
        let work: Work = from_str(&envelope.command.payload).unwrap();
        if work.id == 2 && !self.failed.swap(true, Ordering::SeqCst) {
            return Err(CommandError::Storage("transient".into()));
        }
        self.rows
            .lock()
            .unwrap()
            .entry(envelope.metadata.as_ref().unwrap().command_id)
            .or_insert_with(|| envelope.clone());
        Ok(())
    }
}
#[test]
fn partial_fanout_replays_without_duplicates_and_preserves_metadata() {
    for correlation_id in [Some(Uuid::now_v7()), None] {
        assert_replay_preserves_metadata(false, correlation_id);
    }
}
#[test]
fn envelope_command_fanout_replays_without_duplicates_and_preserves_metadata() {
    for correlation_id in [Some(Uuid::now_v7()), None] {
        assert_replay_preserves_metadata(true, correlation_id);
    }
}
fn assert_replay_preserves_metadata(envelope_reaction: bool, correlation_id: Option<Uuid>) {
    let store = Arc::new(Store::default());
    let gateway = Arc::new(CommandGateway::two_phased(store.clone()));
    let reaction: Box<dyn EventSubscriberPort> = if envelope_reaction {
        Box::new(CommandReaction {
            gateway,
            reaction: Arc::new(|envelope: &EventEnvelope<Batch>| {
                Ok::<_, String>(commands(&envelope.event))
            }),
        })
    } else {
        Box::new(KeyedReaction {
            gateway,
            reaction: commands,
        })
    };
    let run_id = Uuid::now_v7();
    let event_id = Uuid::now_v7();
    assert_ne!(correlation_id, Some(run_id));
    let mut envelope = NewEventEnvelope {
        event_type: Batch::NAME.into(),
        payload: serde_json_to_string(&Batch {
            run_id,
            ids: vec![1, 2, 3],
        })
        .unwrap(),
        metadata: Some(NewEventMetadata {
            sequence: None,
            event_id,
            correlation_id,
            causation_id: None,
            source: Some("test".into()),
        }),
    };
    assert!(reaction.handle(&envelope).is_err());
    assert_eq!(store.rows.lock().unwrap().len(), 1);
    reaction.handle(&envelope).unwrap();
    reaction.handle(&envelope).unwrap();
    assert_eq!(store.rows.lock().unwrap().len(), 3);
    for row in store.rows.lock().unwrap().values() {
        let meta = row.metadata.as_ref().unwrap();
        let work: Work = from_str(&row.command.payload).unwrap();
        assert_eq!(
            meta.command_id,
            stable_command_id(&run_id, &work.id.to_be_bytes())
        );
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, Some(event_id));
    }
    envelope.metadata = None;
    assert!(reaction.handle(&envelope).is_err());
}
#[test]
fn command_identity_matches_standard_uuid_v5() {
    assert_eq!(
        stable_command_id(&Uuid::NAMESPACE_DNS, b"python.org").to_string(),
        "886313e1-3b8a-5372-9b90-0c9aee199e5d"
    );
}
