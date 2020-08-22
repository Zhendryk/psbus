use crate::rc::Subscriber;
use std::collections::{BTreeMap, HashMap};
use std::rc::Weak;

pub(crate) type SubscriberMap<T, E> = HashMap<T, Vec<Weak<dyn Subscriber<T, E>>>>;
pub(crate) type PrioritySubscriberMap<T, E, P> =
    HashMap<T, BTreeMap<P, Vec<Weak<dyn Subscriber<T, E>>>>>;
