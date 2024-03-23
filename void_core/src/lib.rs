use std::future::Future;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Event {}

pub trait Observer<T: Event> {
    fn update(&self, event: &T);
}

pub trait Subject {
    type E: Event;
    fn attach(&mut self, observer: impl Observer<Self::E> + 'static);
    fn detach(&mut self, observer: impl Observer<Self::E> + 'static);
    fn notify(&self, event: Self::E);
}

pub trait Command {}

pub trait CmdSender<T: Command> {
    fn send(&self, cmd: T) -> impl Future<Output = Result<()>>;
}

pub trait CmdReceiver<T: Command> {
    fn recv(&mut self) -> impl Future<Output = Option<T>>;
}

pub trait System {
    type C: Command;
    type R: CmdReceiver<Self::C>;

    fn run(&mut self, receiver: Self::R) -> impl Future<Output = Result<()>>;
}
