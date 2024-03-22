use crate::gui::GuiRenderer;
use egui_wgpu::wgpu;
use void_core::{Event, EventEmitter, EventListner, Publisher};

#[cfg(not(target_arch = "wasm32"))]
mod native;

pub enum RenderEvent {
    PassComplete,
}

impl Event for RenderEvent {}

pub struct RenderEngine<S, R, U, D>
where
    S: EventEmitter<E = U>,
    R: EventListner<E = D>,
    U: Event,
    D: Event,
{
    context: egui::Context,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    gui_renderer: GuiRenderer,
    event_sender: S,
    event_receiver: R,
}

impl<S, R, U, D> RenderEngine<S, R, U, D>
where
    U: Event,
    D: Event,
    S: EventEmitter<E = U>,
    R: EventListner<E = D>,
{
    pub fn handle_render_event(&mut self, _render_event: RenderEvent) {
        todo!()
    }
}
