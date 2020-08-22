use std::hash::Hash;

/// The response given by a `Subscriber`'s `on_event` method, which can also act as a request to the `EventBus`.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum BusRequest {
    NoActionNeeded,
    Unsubscribe,
    DoNotPropagate,
    UnsubscribeAndDoNotPropagate,
    DispatchFailed,
}
unsafe impl Send for BusRequest {}
unsafe impl Sync for BusRequest {}

/// The end result of the `EventBus`'s `dispatch_event` method, which results in one of the following:
///
///     1. `Stopped`: The event was handled by some subscribers in the list, but propagation was halted before the end of the list.
///     2. `Finished`: The event was handled by every subscriber in the list.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum EventDispatchResult {
    NotNeeded,
    Stopped,
    Finished,
    FinishedWithFailures(u32),
}
unsafe impl Send for EventDispatchResult {}
unsafe impl Sync for EventDispatchResult {}

/// Given a list of subscribers from the `EventBus`, this method runs a closure on every subscriber in that list.
///
/// Each of those subscribers will return a resulting `BusRequest`, which we act on accordingly before returning a final `EventDispatchResult`.
pub(crate) fn execute_bus_requests<T, F>(
    subscribers: &mut Vec<T>,
    mut function: F,
) -> EventDispatchResult
where
    F: FnMut(&T) -> BusRequest,
{
    let mut idx = 0;
    let mut failures = 0;
    loop {
        if idx < subscribers.len() {
            // Run our closure function on each subscriber
            match function(&subscribers[idx]) {
                // A return value of None lets us simply move onto the next subscriber
                BusRequest::NoActionNeeded => idx += 1,
                // The rest are self explanatory
                BusRequest::Unsubscribe => {
                    // swap_remove for O(1) operation
                    subscribers.swap_remove(idx);
                }
                BusRequest::DoNotPropagate => {
                    return EventDispatchResult::Stopped;
                }
                BusRequest::UnsubscribeAndDoNotPropagate => {
                    subscribers.swap_remove(idx);
                    return EventDispatchResult::Stopped;
                }
                BusRequest::DispatchFailed => {
                    failures += 1;
                }
            }
        } else {
            // We've made it to the end of our subscriber list without stopping propagation
            if failures == 0 {
                return EventDispatchResult::Finished;
            } else {
                return EventDispatchResult::FinishedWithFailures(failures);
            }
        }
    }
}

// TODO: execute_parallel_bus_requests
