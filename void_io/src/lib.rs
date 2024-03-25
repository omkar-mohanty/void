#[cfg(not(target_arch = "wasm32"))]
mod native;

mod gui;

use egui::{Context, RawInput};
use std::{fmt::Display, sync::Arc};
use void_core::{ICmdReceiver, ICommand, IEvent, Result, ISubject};
use winit::{event::WindowEvent, event_loop::EventLoop, window::Window};

pub enum IoCmd {
    WindowEvent(winit::event::Event<()>),
}

impl Display for IoCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoCmd::WindowEvent(_) => f.write_str("WindowEvent"),
        }
    }
}

impl ICommand for IoCmd {}

#[derive(Clone)]
pub enum IoEvent {
    Output,
    Input(RawInput),
    Resized { height: u32, width: u32 },
    Redraw,
    Exit,
}

impl IEvent for IoEvent {}

pub struct IoEngine<S, R>
where
    S: ISubject<E = IoEvent>,
    R: ICmdReceiver<IoCmd>,
{
    window: Arc<Window>,
    subject: S,
    receiver: R,
}

impl<S, R> IoEngine<S, R>
where
    S: ISubject<E = IoEvent>,
    R: ICmdReceiver<IoCmd>,
{
    pub fn new(context: Context, window: Arc<Window>, subject: S, receiver: R) -> Self {
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
                self.subject.notify(IoEvent::Resized { height, width });
            }
            WindowEvent::RedrawRequested => {
                self.subject.notify(IoEvent::Redraw);
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
        match cmd {
            WindowEvent(event) => self.handle_event(event)?,
        };
        Ok(())
    }
}
