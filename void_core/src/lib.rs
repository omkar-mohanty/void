use std::{
    future::Future,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use egui::Context;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
pub struct Locked<T>(pub Arc<RwLock<T>>);

impl<T> Locked<T> {
    pub fn new(data: T) -> Self {
        Locked(Arc::new(RwLock::new(data)))
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        self.0.read().unwrap()
    }

    pub fn write(&mut self) -> RwLockWriteGuard<T> {
        self.0.write().unwrap()
    }
}

pub trait IBuilder {
    type Output;

    fn build(self) -> impl Future<Output = Result<Self::Output>> + Send;
}

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
