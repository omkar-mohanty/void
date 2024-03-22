use std::future::Future;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Event {}

pub trait Publisher {
    type E: Event;
    fn subscribe(&mut self, event: Self::E);
    fn unsubscribe(&mut self, event: Self::E);
    fn notify(&self, event: Self::E);
}

pub trait Subscriber<E: Event>: Fn(E) {}

pub trait EventEmitter {
    type E: Event;

    fn send_event(&self, event: Self::E) -> impl Future<Output = Result<()>>;
    fn send_event_blocking(&self, event: Self::E) -> Result<()>;
}

pub trait EventListner {
    type E: Event;

    fn receieve_event(&mut self) -> impl Future<Output = Result<Self::E>>;
    fn receive_event_blocking(&mut self) -> Result<Self::E>;
}

pub trait EventSystem {
    type E: Event;
    type Receiver: EventListner<E = Self::E>;
}

pub trait System {
    type EventUp: Event;
    type EventDown: Event;
    type Receiver: EventListner<E = Self::EventDown>;
    type Sender: EventEmitter<E = Self::EventUp>;

    fn run(
        &mut self,
        func: impl FnOnce(Self::EventDown) -> Self::EventUp,
    ) -> impl Future<Output = Result<()>>;
}
