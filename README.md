# psbus
> Generic Publish / Subscribe model for application messaging

> NOTE: This module was heavily inspired by Lakelezz's (now dormant) hey_listen: https://github.com/Lakelezz/hey_listen

`psbus` allows for any application to implement their own Publish/Subscribe model for easy messaging. `psbus` currently supports the following use cases:
* Single-threaded event dispatch
* Single-threaded + prioritized event dispatch
* Thread-safe / synchronized event dispatch
* Thread-safe / synchronized + prioritized event dispatch

# Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
psbus = "0.1.0"
```

# Example
> Aside from type name differences, the usage is consistent across the board for dispatchers. This example uses a single-threaded, non-prioritized dispatch model.

```rust
use psbus::{types::BusRequest, rc::{Event, Subscriber, Publisher, EventBus}};
use std::rc::Rc;

// Define the categories into which your events will fall
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum TestEventType {
    Variant1,
    Variant2,
}

// Define the events themselves, note that you can use bare, tuple or struct enums all the same!
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum TestEvent {
    ButtonPressed(u32),
    SomeOtherEvent,
}

// Implement the Event trait for your custom event
impl Event<TestEventType> for TestEvent {
    fn category(&self) -> TestEventType {
        match self {
            TestEvent::ButtonPressed(_) => TestEventType::Variant1,
            TestEvent::SomeOtherEvent => TestEventType::Variant2,
            // And more...
        }
    }
}

// Now lets make a test Subscriber...
pub struct TestSubscriber {
    id: Uuid,
}
impl Subscriber<TestEventType, TestEvent> for TestSubscriber {
    fn id(&self) -> &Uuid {
        &self.id
    }

    // ! Although we get a TestEvent enum, it is guaranteed to be only of the TestEventType that we are subscribed to
    fn on_event(&self, event: &ThermiteEvent) -> BusRequest {
        println!("Subscriber {} received event: {:?}", self.id, event);
        // What do we want to tell the bus to do after this subscriber is processed? For now, nothing... see crate::types::BusRequest for more actions
        BusRequest::NoActionNeeded
    }
}

// And a test Publisher...
pub struct TestPublisher {}
impl Publisher<TestEventType, TestEvent> for TestPublisher {
    // You can override the `publish_event` method here if you really want to...
}

// And finally, we define our EventBus type
pub type TestEventBus = EventBus<TestEventType, TestEvent>;

fn main() {
    // Again, what you wrap these datastructures with completely depends on your use case.
    let subscriber = Rc::new(TestSubscriber { id: Uuid::default() });
    let publisher = Rc::new(TestPublisher {});
    let bus = Rc::new(RefCell::new(TestEventBus::default()));

    // Subscribe our subscriber to receive any event falling under the `Variant1` category
    bus.try_borrow_mut().expect("Couldn't borrow event bus as mutable").subscribe(&subscriber, TestEventType::Variant1);

    // Publish an event to be received by the subscriber (NOTE: Call site here may vary depending on how you wrap your datastructures)
    publisher.publish_event(&TestEvent::ButtonPressed(1), &mut bus.try_borrow_mut().expect("Couldn't borrow event bus as mutable"));
}
```
