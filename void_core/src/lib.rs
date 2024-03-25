use std::future::Future;

use egui::Context;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait IGui {
    fn show(&mut self, context: &Context);
}

pub trait IEvent {}

pub trait IObserver<T: IEvent>: Send {
    fn update(&self, event: &T) -> Result<()>;
}

pub trait ISubject: Send {
    type E: IEvent;
    fn attach(&mut self, observer: impl IObserver<Self::E> + 'static);
    fn detach(&mut self, observer: impl IObserver<Self::E> + 'static);
    fn notify(&self, event: Self::E);
}

pub trait ICommand {}

pub trait ICmdSender<T: ICommand>: Send {
    fn send(&self, cmd: T) -> impl Future<Output = Result<()>>;
    fn send_blocking(&self, cmd: T) -> Result<()>;
}

pub trait ICmdReceiver<T: ICommand>: Send {
    fn recv(&mut self) -> impl Future<Output = Option<T>> + Send;
    fn recv_blockding(&mut self) -> Option<T>;
}

pub trait ISystem {
    type C: ICommand;
    fn run(&mut self) -> impl Future<Output = Result<()>> + Send;
    fn run_blocking(&mut self) -> Result<()> {
        Ok(())
    }
}
