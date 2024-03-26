use std::{collections::HashMap, sync::Arc};

use void_core::{IEvent, IObserver, ISubject, Result};
use void_render::{RenderEvent, WindowResource};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

pub struct App<'a> {
    pub window_resource: Arc<WindowResource<'a>>,
    pub subject: AppSubject,
}

impl<'a> App<'a> {
    pub fn run(
        &mut self,
        event_loop: EventLoop<()>,
        mut func: impl FnMut(&Event<()>),
    ) -> Result<()> {
        event_loop.run(|event, ewlt| {
            func(&event);

            match event {
                Event::WindowEvent { window_id, event }
                    if window_id == self.window_resource.window.id() =>
                {
                    self.subject
                        .notify(AppEvent::Window(AppWindowEvent::Update))
                        .unwrap();

                    match event {
                        WindowEvent::CloseRequested => ewlt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            let mut config = self.window_resource.config.clone();
                            config.width = physical_size.width;
                            config.height = physical_size.height;
                            self.window_resource
                                .surface
                                .configure(&self.window_resource.device, &config);
                        }
                        WindowEvent::RedrawRequested => {
                            self.window_resource.window.request_redraw();
                            self.subject
                                .notify(AppEvent::Window(AppWindowEvent::Redraw))
                                .unwrap();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })?;
        Ok(())
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum AppWindowEvent {
    Redraw,
    Resize,
    Close,
    Update,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum AppEvent {
    Window(AppWindowEvent),
    Input,
}

impl IEvent for AppEvent {}

#[derive(Default)]
pub struct AppSubject {
    observers: HashMap<AppEvent, Vec<Box<dyn IObserver<AppEvent>>>>,
}

impl ISubject for AppSubject {
    type E = AppEvent;

    fn attach(&mut self, event: Self::E, observer: impl IObserver<AppEvent> + 'static) {
        let entry = self.observers.entry(event).or_default();
        entry.push(Box::new(observer));
    }

    fn detach(&mut self, _event: Self::E, _observer: impl IObserver<AppEvent>) {}

    fn notify(&self, event: Self::E) -> Result<()> {
        if let Some(observers) = self.observers.get(&event) {
            for obs in observers {
                obs.update(event)?;
            }
        }
        Ok(())
    }
}
