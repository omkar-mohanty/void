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

    let render_engine = RendererEngine::new(Arc::clone(&gpu));

    let model_queue = Arc::new(ArrayQueue::new(1));
    let io_engine = IoEngine::new(Arc::clone(&gpu), Arc::clone(&model_queue));

    let model_db = Arc::new(RwLock::new(ModelDB::default()));
    let thread_pool = Arc::new(ThreadPoolBuilder::new().num_threads(8).build()?);

    let app = App {
        gpu: Arc::clone(&gpu),
        render_engine,
        model_db,
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
