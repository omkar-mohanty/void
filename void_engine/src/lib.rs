use void_core::{Observer, Subject};
use void_io::IoEvent;
use void_render::RenderEvent;
use void_ui::GuiEvent;

pub mod gui;
pub mod io;
pub mod render;

#[derive(Default)]
pub struct GuiEngineSubject {
    observers: Vec<Box<dyn Observer<GuiEvent>>>,
}

impl Subject for GuiEngineSubject {
    type E = GuiEvent;

    fn attach(&mut self, observer: impl Observer<Self::E> + 'static) {
        self.observers.push(Box::new(observer));
    }

    fn detach(&mut self, _observer: impl Observer<Self::E> + 'static) {}

    fn notify(&self, event: Self::E) {
        for obs in &self.observers {
            if let Err(msg) = obs.update(&event) {
                log::info!("GuiEngineSubject : {msg}");
            }
        }
    }
}

#[derive(Default)]
pub struct IoEngineSubject {
    observers: Vec<Box<dyn Observer<IoEvent>>>,
}

impl Subject for IoEngineSubject {
    type E = IoEvent;

    fn attach(&mut self, observer: impl Observer<IoEvent> + 'static) {
        self.observers.push(Box::new(observer));
    }

    fn detach(&mut self, _observer: impl Observer<IoEvent>) {}

    fn notify(&self, event: Self::E) {
        for obs in &self.observers {
            if let Err(msg) = obs.update(&event) {
                log::info!("IoEngineSubject : {msg}");
            }
        }
    }
}

#[derive(Default)]
pub struct RenderEngineSubject {
    observers: Vec<Box<dyn Observer<RenderEvent>>>,
}

impl Subject for RenderEngineSubject {
    type E = RenderEvent;

    fn attach(&mut self, observer: impl Observer<RenderEvent> + 'static) {
        self.observers.push(Box::new(observer));
    }

    fn detach(&mut self, _observer: impl Observer<RenderEvent>) {}

    fn notify(&self, event: Self::E) {
        for obs in &self.observers {
            if let Err(msg) = obs.update(&event) {
                log::info!("RenderEngineSubject : {msg}");
            }
        }
    }
}
