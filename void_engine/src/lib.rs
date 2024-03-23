use void_core::{Observer, Subject};
use void_io::IoEvent;

mod gui;

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
            obs.update(&event)
        }
    }
}
