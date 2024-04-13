use std::sync::{Arc, RwLock};
use void_core::{crossbeam_queue::ArrayQueue, rayon::ThreadPoolBuilder};
use void_engine::App;
use void_gpu::{api::Gpu, model::ModelDB};
use void_io::IoEngine;
use void_render::RendererEngine;
use void_window::{event_loop::EventLoop, window::WindowBuilder, Window};

async fn init<'a>() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(Window(WindowBuilder::new().build(&event_loop)?));
    let size = window.inner_size();
    let gpu = Gpu::new(window, size.width, size.height).await;

    let aspect = size.width as f32 / size.height as f32;

    let render_engine = RendererEngine::new(Arc::clone(&gpu), aspect);

    let model_queue = Arc::new(ArrayQueue::new(1));
    let io_engine = IoEngine::new(Arc::clone(&gpu), Arc::clone(&model_queue));

    let thread_pool = Arc::new(ThreadPoolBuilder::new().num_threads(8).build()?);

    let app = App {
        gpu: Arc::clone(&gpu),
        render_engine,
        thread_pool,
        io_engine,
        model_queue,
    };
    app.run(event_loop).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    init().await?;
    Ok(())
}
