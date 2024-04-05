use std::sync::Arc;
use void_core::threadpool::rayon::ThreadPoolBuilder;
use void_engine::App;
use void_gpu::{api::Gpu, model::ModelDB};
use void_render::RendererEngine;
use void_window::{event::WindowEvent, event_loop::EventLoop, window::WindowBuilder, Window};

async fn init<'a>() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(Window(WindowBuilder::new().build(&event_loop)?));
    let size = window.inner_size();
    let gpu = Gpu::new(window, size.width, size.height).await;

    let render_engine = RendererEngine::new(Arc::clone(&gpu));
    let model_db = Arc::new(ModelDB::default());
    let thread_pool = Arc::new(ThreadPoolBuilder::new().build()?);
    let app = App {
        gpu: Arc::clone(&gpu),
        render_engine,
        model_db,
        thread_pool,
    };
    app.run(event_loop)?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    init().await?;
    Ok(())
}
