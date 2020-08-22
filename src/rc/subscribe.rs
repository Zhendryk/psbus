/*
    ABSTRACT: Definition of a single-thread generic subscriber which can subscribe to an
    intermediary event bus (see bus.rs) which dispatches relevant generic events
    that are published to them by one or more publishers (see publish.rs)
*/
use crate::{rc::Event, types::BusRequest};
use std::hash::Hash;
use uuid::Uuid;

/// A generic, single-thread `Subscriber` which subscribes to an `EventBus` to receive events `E` of category `T`, which are published by a `Publisher`.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categories an event can belong to.
///
/// - `E` is meant to be implemented by the module consumer as an enum, depicting the individual events which exist in the system. See `Event`.
pub trait Subscriber<T, E>
where
    T: Eq + PartialEq + Hash + Clone + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + 'static,
{
    fn id(&self) -> &Uuid;
    fn on_event(&self, event: &E) -> BusRequest;
}
