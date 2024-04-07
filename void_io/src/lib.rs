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

    pub async fn handle_window_event(&self, window_event: &WindowEvent) {
        use WindowEvent::*;
        match window_event {
            DroppedFile(path) => {
                let path = std::path::Path::new(path);
                if !path.exists() {
                    log::error!("{} does not exist",path.to_str().unwrap());
                    return;
                }
                let path = path.to_str().unwrap();
                match void_gpu::io::load_model(path, &self.gpu).await {
                    Ok(model) => {
                        if let Err(_model) = self.model_queue.push(model) {
                            log::error!("Could not send model queue full!");
                        }
                    }
                    Err(msg) => {
                        log::error!("Error {msg}");
                    }
                }
            }
            _ => {}
        }
    }

    pub async fn process_event(&self, event: &Event<()>) {
        use Event::*;
        log::info!("Processing event");
        match event {
            WindowEvent { event, .. } => {
                self.handle_window_event(event).await;
            }
            _ => {}
        }
    }

    pub fn run(&self, event: Event<()>) {
        futures::executor::block_on(self.process_event(&event));
    }
}
