use egui::Context;
use void_core::Event;
use winit::window::Window;

pub trait Gui: FnOnce(&Context) {}

#[derive(PartialEq, Eq)]
pub enum IoEvent {}

impl Event for IoEvent {}

pub trait Io {
    type G: Gui;
}

pub struct HidIo<G: Gui> {
    context: Context,
    state: egui_winit::State,
    gui: G,
}

impl<G: Gui> HidIo<G> {
    pub fn new(context: Context, window: Window, gui: G) -> Self {
        let viewport_id = context.viewport_id();
        let state = egui_winit::State::new(context.clone(), viewport_id, &window, None, None);
        Self {
            context,
            state,
            gui,
        }
    }

    pub async fn run(&mut self) {}
}
