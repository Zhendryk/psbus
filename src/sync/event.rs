/*
    ABSTRACT: Definition thread-safe generic events to be handled by their respective publishers, subscribers, and event buses.
*/
use std::hash::Hash;

/// A generic, thread-safe `Event`, categorized by an enum category `T`.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categorie(s) an event can belong to.
///
/// ### Example
///
/// ```rust
/// // TestEventType == T
/// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// pub enum TestEventType {
///     Input,
///     Window,
/// }
/// unsafe impl Send for TestEventType {}
/// unsafe impl Sync for TestEventType {}
///
/// // TestEvent == E
/// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// pub enum TestEvent {
///     Keyboard(KeyboardEvent),
///     Mouse(MouseEvent),
/// }
/// unsafe impl Send for TestEvent {}
/// unsafe impl Sync for TestEvent {}
///
/// impl Event<TestEventType> for TestEvent {
///     fn category(&self) -> TestEventType {
///         match self {
///             TestEvent::Keyboard(_) => TestEventType::Input,
///             TestEvent::Mouse(_) => TestEventType::Input,
///             // And more...
///         }
///     }
/// }
/// ```
pub trait Event<T>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    fn category(&self) -> T;
}
