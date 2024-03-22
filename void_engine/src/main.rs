use egui::Context;
use void_core::System;
use void_native::create_mpsc_channel;
use void_render::RenderEngine;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

fn init() -> anyhow::Result<(Window, Context)> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop)?;

    let context = egui::Context::default();

    Ok((window, context))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (window, context) = init()?;
    let (_app_event_sender, app_event_receiver) = create_mpsc_channel();
    let (engine_sender, _engine_receiver) = create_mpsc_channel();
    let mut render_engine =
        RenderEngine::new(context, &window, app_event_receiver, engine_sender).await;
    let _ = render_engine
        .run(|_app_event| void_render::RenderEvent::PassComplete)
        .await;
    Ok(())
}
