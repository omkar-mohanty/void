use crate::gui::GuiRenderer;
use egui_wgpu::wgpu;
use void_core::{Event, EventReceiver, EventSender, Result, SubSystem};

#[cfg(not(target_arch = "wasm32"))]
mod native;

pub enum RenderEvent {
    PassComplete,
}

impl Event for RenderEvent {}

impl<S, R, P, D> SubSystem<D, RenderEvent, P> for RenderEngine<S, R, RenderEvent, D>
where
    D: Event,
    S: EventSender<E = RenderEvent>,
    R: EventReceiver<E = D>,
    P: FnOnce(D) -> RenderEvent,
{
    async fn run(&mut self, func: P) -> Result<()> {
        let event = self.event_receiver.receieve_event().await?;
        let render_event = func(event);
        self.handle_render_event(render_event);
        Ok(())
    }
}

pub struct RenderEngine<S, R, U, D>
where
    S: EventSender<E = U>,
    R: EventReceiver<E = D>,
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
    S: EventSender<E = U>,
    R: EventReceiver<E = D>,
{
    pub fn handle_render_event(&mut self, _render_event: RenderEvent) {
        todo!()
    }
}
