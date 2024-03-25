use anyhow::Ok;
use std::sync::Arc;
use void_core::IBuilder;
use void_render::{IRenderer, RendererBuilder, WindowResource};
use void_ui::VoidUi;
use winit::{event::WindowEvent, event_loop::EventLoop, window::WindowBuilder};

async fn init<'a>() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);
    let context = egui::Context::default();

    let window_resource = WindowResource::new(window).await;

    let mut gui_renderer = RendererBuilder::new()
        .set_msaa(1)
        .set_context(context.clone())
        .set_resource(Arc::clone(&window_resource))
        .set_gui(VoidUi {})
        .build()
        .await
        .unwrap();

    event_loop.run(move |event, ewlt| {
        use winit::event::Event;
        match event {
            Event::WindowEvent { window_id, event } if window_resource.window.id() == window_id => {
                if !gui_renderer.input(&event) {
                    match event {
                        WindowEvent::CloseRequested => ewlt.exit(),
                        WindowEvent::RedrawRequested => {
                            log::info!("Redraw Requested");
                            gui_renderer.render_blocking().unwrap()
                        }
                        WindowEvent::Resized(physical_size) => {
                            let width = physical_size.width;
                            let height = physical_size.height;
                            let mut config = window_resource.config.clone();
                            config.height = height;
                            config.width =width;
                            window_resource.surface.configure(&window_resource.device, &config);
                        },
                        _ => {}
                    }
                    gui_renderer.gather_input(&event);
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    init().await?;
    Ok(())
}
