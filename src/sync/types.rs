use crate::sync::Subscriber;
use std::collections::{BTreeMap, HashMap};
use std::sync::{RwLock, Weak};

pub(crate) type SubscriberMap<T, E> = HashMap<T, Vec<Weak<RwLock<dyn Subscriber<T, E>>>>>;
pub(crate) type PrioritySubscriberMap<T, E, P> =
    HashMap<T, BTreeMap<P, Vec<Weak<RwLock<dyn Subscriber<T, E>>>>>>;
