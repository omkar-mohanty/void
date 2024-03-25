#[cfg(not(target_arch = "wasm32"))]
mod native;

use egui::{Context, RawInput};
use std::{fmt::Display, sync::Arc};
use void_core::{CmdReceiver, Command, Event, Result, Subject};
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

impl Command for IoCmd {}

#[derive(Clone)]
pub enum IoEvent {
    Output,
    Input(RawInput),
    Resized { height: u32, width: u32 },
    Redraw,
    Exit,
}

impl Event for IoEvent {}

pub struct IoEngine<S, R>
where
    S: Subject<E = IoEvent>,
    R: CmdReceiver<IoCmd>,
{
    window: Arc<Window>,
    state: egui_winit::State,
    subject: S,
    receiver: R,
}

impl<S, R> IoEngine<S, R>
where
    S: Subject<E = IoEvent>,
    R: CmdReceiver<IoCmd>,
{
    pub fn new(context: Context, window: Arc<Window>, subject: S, receiver: R) -> Self {
        let viewport_id = context.viewport_id();
        let state = egui_winit::State::new(context.clone(), viewport_id, &window, None, None);
        Self {
            state,
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
                    let full_input = self.state.take_egui_input(&self.window);
                    self.subject.notify(IoEvent::Input(full_input));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn start_loop(mut self, event_loop: EventLoop<()>) {}

    fn handle_cmd(&mut self, cmd: IoCmd) -> Result<()> {
        use IoCmd::*;
        match cmd {
            WindowEvent(event) => self.handle_event(event)?,
        };
        Ok(())
    }
}
