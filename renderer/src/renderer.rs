use std::{collections::HashMap, ops::Deref};
use void_core::{Event, System, SystemId};

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

pub struct RenderEngine<T: System> {
    gui_renderer: GuiRenderer,
    sub_systems: HashMap<SystemId, T>,
}

impl<A: System> System for RenderEngine<A> {
    type R = ();
    type S = A;
    type T = RenderEvent;

    fn process_event(&mut self, event: Self::T) -> Self::R {
        use RenderEvent::*;
        match event {
            Gui(gui_event) => {
                let full_output = self.gui_renderer.process_event(gui_event);
                todo!("Process GUI Output");
            }
            Scene => {}
            Other(_event) => {
                todo!()
            }
        }
    }

    fn add_subsystem(&mut self, id: SystemId, system: Self::S) {
        self.sub_systems.insert(id, system);
    }
}
