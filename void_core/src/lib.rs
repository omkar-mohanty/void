use std::ops::Deref;

use thiserror::Error;

pub trait RenderSystem: System {
    fn get_id(&self) -> SystemId {
        SystemId("RenderingSystem")
    }
}

pub trait IoSystem: System {
    fn get_output(&self) -> egui::FullOutput;
    fn get_id(&self) -> SystemId {
        SystemId("IoSystem")
    }
}

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

    fn process_event(&self, _event: Self::T) -> Self::R {}
}

pub trait System {
    type T;
    type S: System;
    type R;

    fn process_event(&self, _event_processor: Self::T)  {
    }
    fn add_subsystem(&mut self, _name: SystemId, _sub_system: Self::S) {}
    fn get_id(&self) -> SystemId {
        SystemId::default()
    }
    fn update(&mut self) {
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("System Error `{0}`")]
    System(String),
    #[error("Resource Error `{0}`")]
    Resource(String),
}
