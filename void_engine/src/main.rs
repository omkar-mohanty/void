use anyhow::Ok;
use std::sync::Arc;
use void_core::{ISubject};
use void_engine::{IoEngineSubject};
use void_io::{IoEngine};
use void_native::{create_mpsc_channel};
use winit::{event_loop::EventLoop, window::WindowBuilder};

async fn init<'a>() -> anyhow::Result<()> {
    use void_engine::render::*;

    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);
    let context = egui::Context::default();

    let (render_cmd_sender, render_cmd_receiver) = create_mpsc_channel();
    let (io_cmd_sender, io_cmd_receiver) = create_mpsc_channel();

    //Render engine event publisher

    // Io Engine event publisher
    let mut io_engine_subject = IoEngineSubject::default();
    
    io_engine_subject.attach(RendererObserver {
        cmd_sender: render_cmd_sender.clone(),
    });

    // Gui Engine event publisher

    let io_engine = IoEngine::new(
        context.clone(),
        Arc::clone(&window),
        io_engine_subject,
        io_cmd_receiver,
    );

    todo!("Run Event Loop, init Model Renderer, GuiRenderer etc")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let app = init().await?;
    Ok(())
}
