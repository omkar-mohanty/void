use std::sync::{Arc, RwLock};

use void_core::{crossbeam_queue::ArrayQueue, db::IDb, rayon::ThreadPool, Result, SystemError};
use void_gpu::{
    api::{Gpu, IGpu},
    model::{Model, ModelDB},
};
use void_io::IoEngine;
use void_render::RendererEngine;
use void_window::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    Window,
};

pub struct App<'a> {
    pub gpu: Arc<Gpu<'a, Window>>,
    pub render_engine: RendererEngine<'a, Window>,
    pub thread_pool: Arc<ThreadPool>,
    pub model_db: Arc<RwLock<ModelDB>>,
    pub model_queue: Arc<ArrayQueue<Model>>,
    pub io_engine: IoEngine<'a, Window>,
}

impl<'a> App<'a> {
    pub async fn run(self, event_loop: EventLoop<()>) -> Result<(), SystemError<()>>
where {
        event_loop
            .run(|event, ewlt| {
                        self.check_resources();

                match &event {
                    Event::WindowEvent { window_id, event }
                        if *window_id == self.gpu.window.id() =>
                    {
                        match &event {
                            WindowEvent::CloseRequested => ewlt.exit(),
                            WindowEvent::Resized(physical_size) => {
                                self.gpu
                                    .window_update(physical_size.width, physical_size.height);
                            }
                            WindowEvent::RedrawRequested => {
                                self.render_engine.render();
                                self.gpu.window.request_redraw();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                };
                self.io_engine.run(event);
            })
            .unwrap();
        Ok(())
    }

    fn check_resources(&self) {
        if let Some(model) = self.model_queue.pop() {
            self.render_engine.add_model(&model);
            self.model_db.write().unwrap().insert(std::iter::once(model));
        }
    }
}
