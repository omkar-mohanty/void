use std::sync::Arc;
use void_core::{IBuilder, IEventSender, ISubject};
use void_engine::{App, AppEvent, AppSubject, AppWindowEvent};
use void_gpu::api::GpuResource;
use void_render::{
    gui::GuiRenderer, scene::ModelRenderer, IRenderer, RenderCmd, RenderSubject, RendererEngine,
};
use void_ui::VoidUi;
use winit::{event::WindowEvent, event_loop::EventLoop, window::WindowBuilder};

async fn init<'a>() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);
    let context = egui::Context::default();
    let size = window.inner_size();
    let window_resource = GpuResource::new(window, size.width, size.height).await;

    let gui_renderer = GuiRenderer::builder()
        .set_msaa(1)
        .set_context(context.clone())
        .set_resource(Arc::clone(&window_resource))
        .set_gui(VoidUi {})
        .build()
        .await
        .unwrap();

    let sub = RenderSubject::default();

    let mut app_subject = AppSubject::default();

    let mut app = App {
        subject: app_subject,
        window_resource,
    };

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    init().await?;
    Ok(())
}
