use std::sync::Arc;

use void_core::{crossbeam_queue::ArrayQueue, rayon::ThreadPool, Result, SystemError};
use void_gpu::{
    api::{Gpu, IGpu},
    model::Model,
};
use void_io::IoEngine;
use void_render::RendererEngine;
use void_window::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

pub struct App {
    pub gpu: Arc<Gpu>,
    pub render_engine: RendererEngine,
    pub thread_pool: Arc<ThreadPool>,
    pub model_queue: Arc<ArrayQueue<Model>>,
    pub io_engine: IoEngine,
}

impl App {
    pub async fn run(mut self, event_loop: EventLoop<()>) -> Result<(), SystemError<()>>
where {
        event_loop
            .run(|event, ewlt| {
                self.check_resources();

                match &event {
                    Event::WindowEvent { event, .. } => match &event {
                        WindowEvent::CloseRequested => ewlt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            self.gpu
                                .window_update(physical_size.width, physical_size.height);
                        }
                        WindowEvent::RedrawRequested => {
                            self.render_engine.render().unwrap();
                            self.gpu.get_window().request_redraw();
                        }
                        _ => {}
                    },
                    _ => {}
                };
                self.io_engine.run(event);
            })
            .unwrap();
        Ok(())
    }

    fn check_resources(&mut self) {
        if let Some(model) = self.model_queue.pop() {
            log::info!("Model Added");
            self.render_engine.add_model(model);
        }
    }
}
