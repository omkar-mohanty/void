use void_core::{Observer, Subject};
use void_io::{IoEngine, IoEvent};
use void_render::{RenderEngine, RenderEvent};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

struct App<'a> {
    io_engine: IoEngine<IoEngineSubject>,
    render_engine: RenderEngine<'a, RenderEngineSubject>,
}

#[derive(Default)]
struct RenderEngineSubject {
    observers: Vec<Box<dyn Observer<RenderEvent>>>,
}

impl Subject for RenderEngineSubject {
    type E = RenderEvent;

    fn attach(&mut self, observer: impl Observer<RenderEvent> + 'static) {
        self.observers.push(Box::new(observer));
    }

    fn detach(&mut self, _observer: impl Observer<RenderEvent>) {}

    fn notify(&self, event: Self::E) {
        for obs in &self.observers {
            obs.update(&event);
        }
    }
}

#[derive(Default)]
struct IoEngineSubject {
    observers: Vec<Box<dyn Observer<IoEvent>>>,
}

struct GuiIoObserver {
    raw_input: egui::RawInput,
}

impl Subject for IoEngineSubject {
    type E = IoEvent;

    fn attach(&mut self, observer: impl Observer<IoEvent> + 'static) {
        self.observers.push(Box::new(observer));
    }

    fn detach(&mut self, _observer: impl Observer<IoEvent>) {}

    fn notify(&self, event: Self::E) {
        for obs in &self.observers {
            obs.update(&event)
        }
    }
}

async fn init<'a>(
    window: &'a Window,
    surface: wgpu::Surface<'a>,
    instance: wgpu::Instance,
) -> anyhow::Result<App<'a>> {
    let context = egui::Context::default();

    let engine_subject = RenderEngineSubject::default();

    let io_engine_subject = IoEngineSubject::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let render_engine = RenderEngine::new(
        context.clone(),
        window.inner_size(),
        surface,
        engine_subject,
        adapter,
    )
    .await;
    let io_engine = IoEngine::new(context.clone(), &window, io_engine_subject);

    Ok(App {
        render_engine,
        io_engine,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop)?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let surface = instance.create_surface(&window)?;

    let mut app = init(&window, surface, instance).await?;

    app.io_engine.start_loop(event_loop, &window);

    Ok(())
}
