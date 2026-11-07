use super::tests::{create, input};
use super::*;
use std::cell::RefCell;
use std::rc::Rc;
use testimony::prelude::{
    EventPublisher, EventPublisherError, EventSourcedRepository, EventStore, EventStoreError,
    RepositoryError,
};
use testimony_infra::prelude::InMemoryEventStore;

type Envelope = EventEnvelope<RetailerOfferEvent>;

fn snapshot(events: &[Envelope]) -> Vec<(u64, String, serde_json::Value)> {
    events
        .iter()
        .map(|event| {
            (
                event.sequence(),
                format!("{:?}", event.recorded_at()),
                serde_json::to_value(event.event()).unwrap(),
            )
        })
        .collect()
}

fn update(aggregate: &mut RetailerOfferAggregate) {
    let mut request = input();
    request.content.title = "Updated title".into();
    aggregate.update(
        request.identifiers,
        request.target_market,
        request.language,
        request.content,
        request.price,
        request.availability,
        request.item_condition,
        request.created_at,
    );
}

#[test]
fn commit_preserves_state_and_sequence_and_update_starts_a_new_batch() {
    let mut aggregate = create(input());
    update(&mut aggregate);
    let expected = snapshot(&aggregate.uncommitted_events);
    assert_eq!(snapshot(aggregate.uncommitted_events()), expected);
    assert_eq!(snapshot(aggregate.uncommitted_events()), expected);
    let history = aggregate.uncommitted_events().to_vec();
    let mut replay =
        RetailerOfferAggregate::from_history(aggregate.state.id.clone(), history).unwrap();
    let sequence = aggregate.sequence;
    for _ in 0..2 {
        aggregate.mark_events_committed();
        replay.mark_events_committed();
        assert!(aggregate.uncommitted_events().is_empty());
        assert!(replay.uncommitted_events().is_empty());
        assert_eq!(aggregate.state, replay.state);
        assert_eq!(aggregate.sequence, sequence);
        assert_eq!(replay.sequence, sequence);
    }
    update(&mut aggregate);
    assert_eq!(aggregate.uncommitted_events().len(), 1);
    assert_eq!(aggregate.uncommitted_events()[0].sequence(), sequence + 1);
}

struct Store {
    inner: InMemoryEventStore<RetailerOfferId, RetailerOfferEvent>,
    calls: Rc<RefCell<Vec<&'static str>>>,
    fail: bool,
}
impl EventStore<RetailerOfferId, RetailerOfferEvent> for Store {
    fn load(&self, id: RetailerOfferId) -> Result<Vec<Envelope>, EventStoreError> {
        self.inner.load(id)
    }
    fn append(&self, events: Vec<Envelope>) -> Result<(), EventStoreError> {
        self.calls.borrow_mut().push("append");
        if self.fail {
            return Err(err_append());
        }
        self.inner.append(events)
    }
}
struct Publisher {
    events: Rc<RefCell<Vec<serde_json::Value>>>,
    calls: Rc<RefCell<Vec<&'static str>>>,
    fail: bool,
}
impl EventPublisher<RetailerOfferEvent> for Publisher {
    fn publish(&self, events: &[RetailerOfferEvent]) -> Result<(), EventPublisherError> {
        self.calls.borrow_mut().push("publish");
        if self.fail {
            return Err(err_publish());
        }
        self.events.borrow_mut().extend(
            events
                .iter()
                .map(|event| serde_json::to_value(event).unwrap()),
        );
        Ok(())
    }
}
fn err_append() -> EventStoreError {
    EventStoreError::AppendFailed("test append failure".into())
}
fn err_publish() -> EventPublisherError {
    EventPublisherError::PublishFailed("test publication failure".into())
}

#[test]
fn repository_clears_only_after_success_and_saves_only_new_events() {
    for (fail_append, fail_publish) in [(false, false), (true, false), (false, true)] {
        let inner = InMemoryEventStore::new();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let published = Rc::new(RefCell::new(Vec::new()));
        let store = Store {
            inner: inner.clone(),
            calls: calls.clone(),
            fail: fail_append,
        };
        let publisher = Publisher {
            events: published.clone(),
            calls: calls.clone(),
            fail: fail_publish,
        };
        let repository = EventSourcedRepository::with_publisher(store, publisher);
        let mut aggregate = create(input());
        let expected = snapshot(aggregate.uncommitted_events());
        let result = repository.save(&mut aggregate);
        if fail_append || fail_publish {
            assert_eq!(snapshot(aggregate.uncommitted_events()), expected);
            assert!(published.borrow().is_empty());
            if fail_append {
                assert!(matches!(result, Err(RepositoryError::EventStore(_))));
                assert_eq!(*calls.borrow(), vec!["append"]);
                assert_eq!(inner.total_events(), 0);
            } else {
                assert!(matches!(result, Err(RepositoryError::Publisher(_))));
                assert_eq!(*calls.borrow(), vec!["append", "publish"]);
                assert_eq!(
                    snapshot(&inner.load(aggregate.state.id.clone()).unwrap()),
                    expected
                );
            }
            continue;
        }
        result.unwrap();
        assert!(aggregate.uncommitted_events().is_empty());
        assert_eq!(*calls.borrow(), vec!["append", "publish"]);
        assert_eq!(published.borrow()[0], expected[0].2);
        assert_eq!(
            snapshot(&inner.load(aggregate.state.id.clone()).unwrap()),
            expected
        );
        repository.save(&mut aggregate).unwrap();
        assert_eq!(*calls.borrow(), vec!["append", "publish"]);
        update(&mut aggregate);
        let new_count = aggregate.uncommitted_events().len();
        repository.save(&mut aggregate).unwrap();
        assert!(aggregate.uncommitted_events().is_empty());
        assert_eq!(inner.total_events(), 1 + new_count);
        assert_eq!(published.borrow().len(), 1 + new_count);
        let loaded = repository
            .load(aggregate.state.id.clone())
            .unwrap()
            .unwrap();
        assert_eq!(loaded.state, aggregate.state);
        assert_eq!(loaded.sequence, aggregate.sequence);
        assert!(loaded.uncommitted_events().is_empty());
    }
}
