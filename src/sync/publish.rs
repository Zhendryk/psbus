/*
    ABSTRACT: Definition of a thread-safe generic publisher which utilizes an intermediary event bus
    (see bus.rs) to send generic messages to its respective subscribers (see subscribe.rs)
*/
use crate::{
    sync::{Event, EventBus},
    types::EventDispatchResult,
};
use std::hash::Hash;

/// A generic, thread-safe `Publisher` which publishes events `E` of category `T` to a list of `Subscribers` via an `EventBus`.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categories an event can belong to.
///
/// - `E` is meant to be implemented by the module consumer as an enum, depicting the individual events which exist in the system. See `Event`.
pub trait Publisher<T, E>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    fn publish_event(&self, event: &E, bus: &mut EventBus<T, E>) -> EventDispatchResult {
        bus.dispatch_blocking_event(event)
    }
}
