/*
    ABSTRACT: Definition of single-thread event bus datastructure and its supporting
    datatypes to delegate events (see event.rs) between publishers
    (see publish.rs) and subscribers (see subscribe.rs)
*/
#![allow(dead_code)]
use crate::{
    rc::{types::*, Event, Subscriber},
    types::*,
};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::rc::{Rc, Weak};

/// Single-thread datastructure responsible for dispatching events from `Publisher`s to `Subscriber`s
///
/// This keeps the respective Pub/Sub systems decoupled from each other
///
/// This should be wrapped in a Rc<RefCell<EventBus>>
pub struct EventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + 'static,
{
    // We hold a std::rc::Weak (Rc which holds non-owning reference) to not prevent dropping and to avoid circular references to an Rc
    // We can deal with subscribers that get dropped by just removing them from our map if we find they did get dropped
    channels: SubscriberMap<T, E>,
}

impl<T, E> Default for EventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + 'static,
{
    fn default() -> Self {
        Self {
            channels: HashMap::default(),
        }
    }
}

impl<T, E> EventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + 'static,
{
    /// Adds the given `Subscriber` to a subscriber list to receive published messages of the given event category
    pub fn subscribe<S: Subscriber<T, E> + 'static>(&mut self, subscriber: &Rc<S>, to_category: T) {
        if let Some(subscriber_list) = self.channels.get_mut(&to_category) {
            // We have an existing subscriber list for this category, push a new subscriber to it
            subscriber_list.push(Rc::downgrade(
                &(subscriber.clone() as Rc<dyn Subscriber<T, E> + 'static>),
            ));
        } else {
            // No subscriber list exists yet for this category, create one
            self.channels.insert(
                to_category,
                vec![Rc::downgrade(
                    &(subscriber.clone() as Rc<dyn Subscriber<T, E> + 'static>),
                )],
            );
        }
    }

    /// Unsubscribes the given `Subscriber` from the given category on this `EventBus`
    ///
    /// ### Notes
    /// - Automatically removes any dropped subscribers in the channel corresponding to the given category, if the bus encounters any.
    pub fn unsubscribe<S: Subscriber<T, E> + 'static>(&mut self, subscriber: &S, from_category: T) {
        let mut cleanup_required = false;
        if let Some(subscriber_list) = self.channels.get_mut(&from_category) {
            if let Some(idx) = subscriber_list.iter().position(|weak_sub| {
                if let Some(sub) = weak_sub.upgrade() {
                    sub.id() == subscriber.id()
                } else {
                    // We dropped a subscriber, need to clean up
                    cleanup_required = true;
                    false
                }
            }) {
                // We can swap_remove for O(1) performance here because we don't care about ordering
                subscriber_list.swap_remove(idx);
            }

            if cleanup_required {
                subscriber_list.retain(|susbcriber| Weak::clone(susbcriber).upgrade().is_some());
            }
        }
    }

    /// Removes all `Subscriber`s from this `EventBus`
    ///
    /// ### Notes
    /// - The memory previously allocated for the `Subscriber`s remains allocated for reuse.
    pub fn unsubscribe_all(&mut self) {
        self.channels.clear()
    }

    /// Removes all `Subscriber`s from the given category on this `EventBus`
    pub fn unsubscribe_all_from_category(&mut self, from_category: T) {
        self.channels.remove(&from_category);
    }

    /// Dispatches the given event to all `Subscriber`s of that event's category
    ///
    /// ### Notes
    /// - Automatically removes any dropped `Subscriber`s in the channel the given event belongs to, if the bus encounters any.
    pub fn dispatch_event(&mut self, event: &E) -> EventDispatchResult {
        let mut result = EventDispatchResult::NotNeeded;
        // Grab our list of subscribers for this event's category, if one exists
        if let Some(subscriber_list) = self.channels.get_mut(&event.category()) {
            let mut cleanup_required = false;
            // Attempt to have all subscribers handle the dispatched event and return requests to the event bus (non-blocking)
            result = execute_bus_requests(subscriber_list, |weak_subscriber| {
                if let Some(subscriber) = weak_subscriber.upgrade() {
                    subscriber.on_event(event)
                } else {
                    // Found an invalid reference to a subscriber (which was probably dropped by the owner)
                    cleanup_required = true;
                    BusRequest::NoActionNeeded
                }
            });

            if cleanup_required {
                subscriber_list.retain(|susbcriber| Weak::clone(susbcriber).upgrade().is_some());
            }
        }

        result
    }
}

/// Single-thread datastructure responsible for dispatching events from `Publisher`s to `Subscriber`s in a prioritized order
///
/// This keeps the respective Pub/Sub systems decoupled from each other
///
/// This should be wrapped in a Rc<RefCell<PriorityEventBus>>
pub struct PriorityEventBus<T, E, P>
where
    T: Eq + PartialEq + Hash + Clone + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + 'static,
    P: Ord,
{
    channels: PrioritySubscriberMap<T, E, P>,
}

impl<T, E, P> Default for PriorityEventBus<T, E, P>
where
    T: Eq + PartialEq + Hash + Clone + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + 'static,
    P: Ord,
{
    fn default() -> Self {
        Self {
            channels: HashMap::default(),
        }
    }
}

impl<T, E, P> PriorityEventBus<T, E, P>
where
    T: Eq + PartialEq + Hash + Clone + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + 'static,
    P: Ord,
{
    /// Adds the given `Subscriber` to a prioritized subscriber list to receive published messages of the given event category
    pub fn subscribe<S: Subscriber<T, E> + 'static>(
        &mut self,
        subscriber: &Rc<S>,
        to_category: T,
        with_priority: P,
    ) {
        if let Some(category_priority_map) = self.channels.get_mut(&to_category) {
            if let Some(subscriber_list) = category_priority_map.get_mut(&with_priority) {
                // We have an existing subscriber list for this category, push a new subscriber to it
                subscriber_list.push(Rc::downgrade(
                    &(subscriber.clone() as Rc<dyn Subscriber<T, E> + 'static>),
                ));
            } else {
                // No subscriber list exists yet for this priority segment in this category, create one
                category_priority_map.insert(
                    with_priority,
                    vec![Rc::downgrade(
                        &(subscriber.clone() as Rc<dyn Subscriber<T, E> + 'static>),
                    )],
                );
            }
        } else {
            // This category doesn't exist yet, create it
            let mut priority_map = BTreeMap::default();
            priority_map.insert(
                with_priority,
                vec![Rc::downgrade(
                    &(subscriber.clone() as Rc<dyn Subscriber<T, E> + 'static>),
                )],
            );
            self.channels.insert(to_category, priority_map);
        }
    }

    /// Unsubscribes the given `Subscriber` from the given priority segment in the given category from this `PriorityEventBus`
    ///
    /// ### Notes
    /// - This method drills down to the provided category and priority segment within that category directly to locate and unsubscribe a `Subscriber`.
    /// - This method automatically removes any dropped subscribers it encounters during the search.
    ///
    /// ### Returns
    /// - `bool`: `true` if the subscriber was successfully unsubscribed, `false` if it was not (for various reasons, including not found).
    pub fn unsubscribe<S: Subscriber<T, E> + 'static>(
        &mut self,
        subscriber: &S,
        from_category: &T,
        with_priority: &P,
    ) -> bool {
        // Grab our priority map
        if let Some(category_priority_map) = self.channels.get_mut(from_category) {
            // Grab the subscriber list and find the index of the subscriber to unsubscribe
            if let Some(subscriber_list) = category_priority_map.get_mut(with_priority) {
                let mut cleanup_required = false;
                if let Some(idx) = subscriber_list.iter().position(|weak_sub| {
                    if let Some(sub) = weak_sub.upgrade() {
                        sub.id() == subscriber.id()
                    } else {
                        // Found an invalid reference to a subscriber (which was probably dropped by the owner)
                        cleanup_required = true;
                        false
                    }
                }) {
                    // We can swap_remove here because the subscribers are pooled by priority, and the ordering of two subscribers with the same priority doesn't matter
                    subscriber_list.swap_remove(idx);
                    return true;
                }
                if cleanup_required {
                    subscriber_list
                        .retain(|susbcriber| Weak::clone(susbcriber).upgrade().is_some());
                }
            }
        }
        false
    }

    /// Removes all `Subscriber`s from this `PriorityEventBus`
    ///
    /// ### Notes
    /// - The memory previously allocated for the `Subscriber`s remains allocated for reuse.
    pub fn unsubscribe_all(&mut self) {
        self.channels.clear()
    }

    /// Removes all `Subscriber`s from the given category on this `PriorityEventBus`
    pub fn unsubscribe_all_from_category(&mut self, from_category: &T) {
        self.channels.remove(from_category);
    }

    /// Removes all `Subscriber`s from the given priority segment in the given category on this `PriorityEventBus`
    pub fn unsubscribe_all_from_category_prioritized(
        &mut self,
        from_category: &T,
        with_priority: &P,
    ) {
        if let Some(category_priority_map) = self.channels.get_mut(from_category) {
            category_priority_map.remove(with_priority);
        }
    }

    /// Dispatches the given event to all `Subscriber`s of that event's category
    ///
    /// ### Notes
    /// - Automatically removes any dropped `Subscriber`s in the channel the given event belongs to, if the bus encounters any.
    pub fn dispatch_event(&mut self, event: &E) -> EventDispatchResult {
        let mut result = EventDispatchResult::NotNeeded;
        // Grab the priority map for our category
        if let Some(category_priority_map) = self.channels.get_mut(&event.category()) {
            // For each distinct priority segment, in order of priority
            for subscriber_list in category_priority_map.values_mut() {
                // Dispatch the event, automatically stopping propagation via `execute_bus_requests` if necessary
                let mut cleanup_required = false;
                result = execute_bus_requests(subscriber_list, |weak_subscriber| {
                    if let Some(subscriber) = weak_subscriber.upgrade() {
                        subscriber.on_event(event)
                    } else {
                        // Found an invalid reference to a subscriber (which was probably dropped by the owner)
                        cleanup_required = true;
                        BusRequest::NoActionNeeded
                    }
                });
                if cleanup_required {
                    subscriber_list
                        .retain(|subscriber| Weak::clone(subscriber).upgrade().is_some());
                }
            }
        }
        result
    }
}

// TODO: Add ParallelEventBus

// TODO: Add testing
