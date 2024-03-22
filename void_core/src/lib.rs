use std::future::Future;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Event {}

pub trait EventSender {
    type E: Event;

    fn send_event(&self, event: Self::E) -> impl Future<Output = Result<()>>;
    fn send_event_blocking(&self, event: Self::E) -> Result<()>;
}

pub trait EventReceiver {
    type E: Event;

    fn receieve_event(&mut self) -> impl Future<Output = Result<Self::E>>;
    fn receive_event_blocking(&mut self) -> Result<Self::E>;
}

pub trait System {
    type EventUp: Event;
    type EventDown: Event;
    type Receiver: EventReceiver<E = Self::EventDown>;
    type Sender: EventSender<E = Self::EventUp>;
}

pub trait SubSystem<S, D, P>
where
    S: Event,
    D: Event,
    P: FnOnce(S) -> D,
{
    fn run(&mut self, func: P) -> impl Future<Output = Result<()>>;
}
