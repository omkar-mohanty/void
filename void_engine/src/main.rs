use std::sync::Arc;
use void_core::{IBuilder, IEventSender, ISubject, ISystem};
use void_engine::{App, AppEvent, AppSubject, AppWindowEvent};
use void_native::create_mpsc_channel;
use void_render::{
    gui::GuiRenderer, scene::ModelRenderer, IRenderer, RenderCmd, RenderSubject, RendererEngine,
    WindowResource,
};
use void_ui::VoidUi;
use winit::{event::WindowEvent, event_loop::EventLoop, window::WindowBuilder};

async fn init<'a>() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);
    let context = egui::Context::default();

    let window_resource = WindowResource::new(window).await;

    let mut gui_renderer = GuiRenderer::builder()
        .set_msaa(1)
        .set_context(context.clone())
        .set_resource(Arc::clone(&window_resource))
        .set_gui(VoidUi {})
        .build()
        .await
        .unwrap();

    let sub = RenderSubject::default();

    let (send, recv) = create_mpsc_channel();

    let mut model_renderer = ModelRenderer::builder()
        .set_resource(Arc::clone(&window_resource))
        .set_subject(sub)
        .set_receiver(recv)
        .build()
        .await
        .unwrap();

    let mut render_engine =
        RendererEngine::new(gui_renderer, model_renderer, Arc::clone(&window_resource));

    let mut app_subject = AppSubject::default();
    app_subject.attach(AppEvent::Window(AppWindowEvent::Redraw), move || {
        send.clone().send_blocking(RenderCmd::Render)
    });

    let mut app = App {
        subject: app_subject,
        window_resource,
    };

    app.run(event_loop, |event| {
        use winit::event::Event;
        match event {
            Event::WindowEvent { event, .. } => {
                render_engine.gather_input(event);
                match event {
                    WindowEvent::RedrawRequested => {
                        log::info!("Redraw");
                        if let Err(msg) = render_engine.render_blocking() {
                            log::error!("{msg}");
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    })
    .unwrap();

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    init().await?;
    Ok(())
}
