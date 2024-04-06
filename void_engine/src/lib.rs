use std::sync::Arc;

use void_core::{rayon::ThreadPool, Result, SystemError};
use void_gpu::{
    api::{Gpu, IGpu},
    model::ModelDB,
};
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
    pub model_db: Arc<ModelDB>,
}

impl<'a> App<'a> {
    pub fn run(self, event_loop: EventLoop<()>) -> Result<(), SystemError<()>>
where {
        event_loop
            .run(|event, ewlt| match event {
                Event::WindowEvent { window_id, event } if window_id == self.gpu.window.id() => {
                    match event {
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
            })
            .unwrap();
        Ok(())
    }
}
