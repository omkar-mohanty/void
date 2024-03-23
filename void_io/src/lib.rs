use egui::{Context, RawInput};
use void_core::{Event, Subject};
use winit::{
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::Window,
};

#[derive(Clone)]
pub enum IoEvent {
    Output,
    Input(RawInput),
    Resized { height: u32, width: u32 },
    Exit,
}

impl Event for IoEvent {}

pub struct IoEngine<S>
where
    S: Subject<E = IoEvent>,
{
    context: Context,
    state: egui_winit::State,
    subject: S,
}

impl<S> IoEngine<S>
where
    S: Subject<E = IoEvent>,
{
    pub fn new(context: Context, window: &Window, subject: S) -> Self {
        let viewport_id = context.viewport_id();
        let state = egui_winit::State::new(context.clone(), viewport_id, &window, None, None);
        Self {
            context,
            state,
            subject,
        }
    }

    #[allow(unused_variables)]
    fn input(&self, window: &Window, event: &WindowEvent) -> bool {
        window.request_redraw();
        false
    }

    fn handle_window_event(&self, window_event: WindowEvent, ewlt: &EventLoopWindowTarget<()>) {
        match window_event {
            WindowEvent::CloseRequested => {
                ewlt.exit();
            }
            WindowEvent::Resized(physical_size) => {
                let (height, width) = (physical_size.height, physical_size.width);
                self.subject.notify(IoEvent::Resized { height, width });
            }
            _ => {}
        }
    }

    pub fn start_loop(&mut self, event_loop: EventLoop<()>, window: &Window) {
        let _ = event_loop.run(move |event, ewlt| {
            use winit::event::Event;
            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    if !self.input(window, &event) {
                        self.handle_window_event(event, ewlt);
                        let full_input = self.state.take_egui_input(window);
                        self.subject.notify(IoEvent::Input(full_input));
                    }
                }
                _ => {}
            }
        });
    }
}
