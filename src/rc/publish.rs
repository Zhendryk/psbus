/*
    ABSTRACT: Definition of a single-thread generic publisher which utilizes an intermediary event bus
    (see bus.rs) to send generic messages to its respective subscribers (see subscribe.rs)
*/
use crate::{
    rc::{Event, EventBus},
    types::EventDispatchResult,
};
use std::hash::Hash;

/// A generic, single-thread `Publisher` which publishes events `E` of category `T` to a list of `Subscribers` via an `EventBus`.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categories an event can belong to.
///
/// - `E` is meant to be implemented by the module consumer as an enum, depicting the individual events which exist in the system. See `Event`.
pub trait Publisher<T, E>
where
    T: Eq + PartialEq + Hash + Clone + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + 'static,
{
    fn publish_event(&self, event: &E, bus: &mut EventBus<T, E>) -> EventDispatchResult {
        bus.dispatch_event(event)
    }
}
