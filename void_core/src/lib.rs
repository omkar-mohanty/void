#![feature(fn_traits)]

use std::{future::Future, hash::Hash};

use egui::Context;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait IBuilder {
    type Output;

    fn build(self) -> impl Future<Output = Result<Self::Output>> + Send;
}

pub trait IGui {
    fn show(&mut self, context: &Context);
}

pub trait IEvent: Hash + Clone + Copy + Eq {}

pub trait IObserver<T: IEvent + Hash + Clone + Copy + Eq>: Send {
    fn update(&self, event: T) -> Result<()>;
}

impl<T, E> IObserver<E> for T
where
    E: IEvent,
    T: Fn() -> Result<()> + Send,
{
    fn update(&self, _event: E) -> Result<()> {
        self.call(())?;
        Ok(())
    }
}

pub trait ISubject: Send {
    type E: IEvent;
    fn attach(&mut self, event: Self::E, observer: impl IObserver<Self::E> + 'static);
    fn detach(&mut self, event: Self::E, observer: impl IObserver<Self::E> + 'static);
    fn notify(&self, event: Self::E) -> Result<()>;
}

pub trait IEventSender<T: IEvent>: Send {
    fn send(&self, cmd: T) -> impl Future<Output = Result<()>>;
    fn send_blocking(&self, cmd: T) -> Result<()>;
}

pub trait IEventReceiver<T: IEvent>: Send {
    fn recv(&mut self) -> impl Future<Output = Option<T>> + Send;
    fn recv_blockding(&mut self) -> Option<T>;
}

pub trait ISystem {
    type C: IEvent;
    fn run(&mut self) -> impl Future<Output = Result<()>> + Send;
    fn run_blocking(&mut self) -> Result<()> {
        Ok(())
    }
}
