#[cfg(not(target_arch = "wasm32"))]
mod native;

mod gui;

use std::{fmt::Display, sync::Arc};
use void_core::{IEvent, IEventReceiver, ISubject, Result};
use winit::{event::WindowEvent, window::Window};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum IoCmd {}

impl IEvent for IoCmd {}

#[derive(Clone, Hash, Copy, PartialEq, Eq)]
pub enum AppEvent {
    Output,
    Input,
    Resized,
    Redraw,
    Exit,
}

impl IEvent for AppEvent {}

pub struct IoEngine<S, R>
where
    S: ISubject<E = AppEvent>,
    R: IEventReceiver<IoCmd>,
{
    window: Arc<Window>,
    subject: S,
    receiver: R,
}

impl<S, R> IoEngine<S, R>
where
    S: ISubject<E = AppEvent>,
    R: IEventReceiver<IoCmd>,
{
    pub fn new(window: Arc<Window>, subject: S, receiver: R) -> Self {
        Self {
            subject,
            window,
            receiver,
        }
    }

    #[allow(unused_variables)]
    fn input(&self, window: &Window, event: &WindowEvent) -> bool {
        window.request_redraw();
        false
    }

    fn handle_window_event(&self, window_event: WindowEvent) -> Result<()> {
        match window_event {
            WindowEvent::Resized(physical_size) => {
                let (height, width) = (physical_size.height, physical_size.width);
                self.subject.notify(AppEvent::Resized);
            }
            WindowEvent::RedrawRequested => {
                self.subject.notify(AppEvent::Redraw);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_event(&mut self, event: winit::event::Event<()>) -> Result<()> {
        use winit::event::Event;
        match event {
            Event::WindowEvent { window_id, event } if window_id == self.window.id() => {
                if !self.input(&self.window, &event) {
                    self.handle_window_event(event)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_cmd(&mut self, cmd: IoCmd) -> Result<()> {
        use IoCmd::*;

        Ok(())
    }
}
