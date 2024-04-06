#![feature(fn_traits)]

use std::future::Future;

use egui::Context;
mod error;

pub mod db;
mod threadpool;
pub use error::*;
pub use threadpool::*;

pub type Result<T, E> = std::result::Result<T, E>;

pub trait IBuilder {
    type Output;

    fn build(self) -> impl Future<Output = Result<Self::Output, BuilderError>> + Send;
}

pub trait IGui {
    fn show(&mut self, context: &Context);
}

pub trait ISystem {
    type Err;

    fn step(&self) -> Result<(), Self::Err>;
}
