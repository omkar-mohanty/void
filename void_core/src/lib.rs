#![feature(fn_traits)]

use std::{
    future::Future,
    hash::Hash,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use egui::Context;
mod error;
mod newtypes;

pub mod db;
pub mod threadpool;
pub use error::*;
pub use newtypes::*;

pub struct Locked<T>(Arc<RwLock<T>>);

impl<T> Locked<T> {
    pub fn new(val: T) -> Self {
        Self(Arc::new(RwLock::new(val)))
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        self.0.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.0.write().unwrap()
    }
}

pub type Result<T, E> = std::result::Result<T, E>;

pub trait IBuilder {
    type Output;

    fn build(self) -> impl Future<Output = Result<Self::Output, BuilderError>> + Send;
}

pub trait IGui {
    fn show(&mut self, context: &Context);
}

pub trait IEvent: Hash + Clone + Copy + Eq {}

pub trait IObserver<T: IEvent + Hash + Clone + Copy + Eq>: Send {
    fn update(&self, event: T) -> Result<(), FutError<T>>;
}

impl<T, E> IObserver<E> for T
where
    E: IEvent,
    T: Fn() -> Result<(), FutError<E>> + Send,
{
    fn update(&self, _event: E) -> Result<(), FutError<E>> {
        self.call(())?;
        Ok(())
    }
}

pub trait ISubject: Send {
    type E: IEvent;
    fn attach(&mut self, event: Self::E, observer: impl IObserver<Self::E> + 'static);
    fn detach(&mut self, event: Self::E, observer: impl IObserver<Self::E> + 'static);
    fn notify(&self, event: Self::E) -> Result<(), FutError<Self::E>>;
}

pub trait IEventSender<T: IEvent>: Send {
    fn send(&self, cmd: T) -> impl Future<Output = std::result::Result<(), FutError<T>>>;
    fn send_blocking(&self, cmd: T) -> std::result::Result<(), FutError<T>>;
}

pub trait IEventReceiver<T: IEvent>: Send {
    fn recv(&mut self) -> impl Future<Output = Option<std::result::Result<T, FutError<T>>>> + Send;
    fn recv_blocking(&mut self) -> Option<std::result::Result<T, FutError<T>>>;
}

pub trait ISystem {
    type C: IEvent;
    fn run(&mut self) -> impl Future<Output = Result<(), SystemError<Self::C>>> + Send;
    fn run_blocking(&mut self) -> Result<(), SystemError<Self::C>> {
        Ok(())
    }
}
