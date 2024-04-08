use std::sync::Arc;

use void_core::crossbeam_queue::ArrayQueue;
use void_gpu::{
    api::{Displayable, Gpu},
    model::Model,
};
use void_window::event::*;

pub struct IoEngine<'a, T: Displayable<'a>> {
    gpu: Arc<Gpu<'a, T>>,
    model_queue: Arc<ArrayQueue<Model>>,
}

impl<'a, T: Displayable<'a>> IoEngine<'a, T> {
    pub fn new(gpu: Arc<Gpu<'a, T>>, model_queue: Arc<ArrayQueue<Model>>) -> Self {
        Self { gpu, model_queue }
    }

    pub fn handle_window_event(&self, window_event: &WindowEvent) {
        use WindowEvent::*;
        match window_event {
            DroppedFile(path) => {
                log::info!("Dropped File");
                match void_gpu::io::load_model(path, &self.gpu) {
                    Ok(model) => {
                        if let Err(_model) = self.model_queue.push(model) {
                            log::error!("Could not send model queue full!");
                        }
                    }
                    Err(msg) => {
                        log::error!("Error: {msg} for {}", path.display());
                    }
                }
            }
            _ => {}
        }
    }

    pub fn process_event(&self, event: &Event<()>) {
        use Event::*;
        match event {
            WindowEvent { event, .. } => {
                self.handle_window_event(event);
            }
            _ => {}
        }
    }

    pub fn run(&self, event: Event<()>) {
        self.process_event(&event);
    }
}
