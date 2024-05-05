use anyhow::Result;
use void::app::App;
use winit::{event_loop::EventLoop, window::WindowBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop)?;
    let mut app = App::new(window).await;
    app.run(event_loop).await;
    Ok(())
}
