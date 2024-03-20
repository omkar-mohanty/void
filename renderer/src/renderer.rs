use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Receiver;
use void_core::{Event, IoSystem, System, SystemId};

use egui_wgpu::wgpu;

use crate::gui::{GuiRenderEvent, GuiRenderer};

impl Event for RenderEvent {
    fn system(&self) -> void_core::SystemId {
        SystemId("RenderEngine")
    }
}

pub enum RenderEvent {
    Gui(GuiRenderEvent),
    Scene,
    Other(Box<dyn Event>),
}

pub struct RenderEngine<T: System, I: IoSystem> {
    context: egui::Context,
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture_view: wgpu::TextureView,
    gui_renderer: GuiRenderer,
    io_system: Arc<I>,
    sub_systems: HashMap<SystemId, T>,
}

impl<A: System, I: IoSystem> System for RenderEngine<A, I> {
    type R = ();
    type S = A;
    type T = Receiver<RenderEvent>;

    fn process_event(&self, mut event: Self::T) -> Self::R {
        if let Some(_event) = event.blocking_recv() {}
    }

    fn add_subsystem(&mut self, id: SystemId, system: Self::S) {
        self.sub_systems.insert(id, system);
    }

    fn update(&mut self) {
        let _full_output = self.io_system.get_output();
        todo!()
    }
}
