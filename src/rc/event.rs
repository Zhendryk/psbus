/*
    ABSTRACT: Definition single-thread generic events to be handled by their respective publishers, subscribers, and event buses.
*/
use std::hash::Hash;

/// A generic, single-thread `Event`, categorized by an enum category `T`.
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
///
/// // TestEvent == E
/// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// pub enum TestEvent {
///     Keyboard(KeyboardEvent),
///     Mouse(MouseEvent),
/// }
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
    // ! NOTE: 'static on trait object means that T does not contain any references with a lifetime less than 'static
    // ! See: https://stackoverflow.com/questions/40053550/the-compiler-suggests-i-add-a-static-lifetime-because-the-parameter-type-may-no
    T: Eq + PartialEq + Hash + Clone + 'static,
{
    fn category(&self) -> T;
}
