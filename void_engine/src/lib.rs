use void_core::{IObserver, ISubject};
use void_io::IoEvent;
use void_render::RenderEvent;

pub mod gui;
pub mod io;
pub mod render;

#[derive(Default)]
pub struct IoEngineSubject {
    observers: Vec<Box<dyn IObserver<IoEvent>>>,
}

impl ISubject for IoEngineSubject {
    type E = IoEvent;

    fn attach(&mut self, observer: impl IObserver<IoEvent> + 'static) {
        self.observers.push(Box::new(observer));
    }

    fn detach(&mut self, _observer: impl IObserver<IoEvent>) {}

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
    observers: Vec<Box<dyn IObserver<RenderEvent>>>,
}

impl ISubject for RenderEngineSubject {
    type E = RenderEvent;

    fn attach(&mut self, observer: impl IObserver<RenderEvent> + 'static) {
        self.observers.push(Box::new(observer));
    }

    fn detach(&mut self, _observer: impl IObserver<RenderEvent>) {}

    fn notify(&self, event: Self::E) {
        for obs in &self.observers {
            if let Err(msg) = obs.update(&event) {
                log::info!("RenderEngineSubject : {msg}");
            }
        }
    }
}
