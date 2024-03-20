use std::ops::Deref;

use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub struct SystemId(pub &'static str);

impl Default for SystemId {
    fn default() -> Self {
        Self("Default")
    }
}

pub trait Event {
    fn system(&self) -> SystemId {
        SystemId::default()
    }
}

impl Event for Box<dyn Event> {
    fn system(&self) -> SystemId {
        self.deref().system()
    }
}

impl Event for () {}

impl System for () {
    type T = ();
    type S = ();
    type R = ();

    fn process_event(&mut self, _event: Self::T) -> Self::R {}

    fn add_subsystem(&mut self, _name: SystemId, _sub_system: Self::S) {}
}

pub trait System {
    type T: Event;
    type S: System;
    type R;

    fn process_event(&mut self, event: Self::T) -> Self::R;
    fn add_subsystem(&mut self, _name: SystemId, _sub_system: Self::S) {}
    fn get_id(&self) -> SystemId {
        SystemId::default()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("System Error `{0}`")]
    System(String),
    #[error("Resource Error `{0}`")]
    Resource(String),
}
