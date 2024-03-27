use core::fmt;
use std::{future::Future, sync::Arc};

use egui::ahash::HashMap;
use void_core::{FutError, IBuilder, IEvent, IEventReceiver, IGui, IObserver, ISubject};
use winit::window::Window;

use self::{gui::GuiRenderer, scene::ModelRenderer};

pub mod gui;
pub mod model;
pub mod pipeline;
pub mod scene;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum RenderCmd {
    Render,
}

pub trait Draw {
    fn draw(&mut self, surface: Arc<wgpu::TextureView>);
}

pub struct RendererEngine<'a, G, P, R>
where
    G: IGui + Send + Default,
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    window_resource: Arc<WindowResource<'a>>,
    gui_renderer: GuiRenderer<'a, G>,
    model_renderer: ModelRenderer<'a, P, R>,
}

impl<'a, G, P, R> RendererEngine<'a, G, P, R>
where
    G: IGui + Send + Default,
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    pub fn new(
        gui_renderer: GuiRenderer<'a, G>,
        model_renderer: ModelRenderer<'a, P, R>,
        window_resource: Arc<WindowResource<'a>>,
    ) -> Self {
        Self {
            gui_renderer,
            model_renderer,
            window_resource,
        }
    }

    pub fn gather_input(&mut self, event: &winit::event::WindowEvent) {
        self.gui_renderer.gather_input(event);
    }
}

impl<'a, G, P, R> IRenderer for RendererEngine<'a, G, P, R>
where
    G: IGui + Send + Default,
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    async fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        unimplemented!()
    }
    fn render_blocking(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        let output = self.window_resource.surface.get_current_texture()?;
        let view = Arc::new(output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        }));
        std::thread::scope(|_| {
            self.model_renderer.draw(Arc::clone(&view));
        });
        std::thread::scope(|_| {
            self.gui_renderer.draw(Arc::clone(&view));
        });
        output.present();
        Ok(())
    }
}

impl fmt::Display for RenderCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RenderCmd::*;
        match self {
            Render => f.write_str("Render"),
        }
    }
}

impl IEvent for RenderCmd {}

#[derive(Hash, Clone, Copy, Eq, PartialEq, Debug)]
pub enum RenderEvent {
    PassComplete,
}

impl IEvent for RenderEvent {}

pub trait IRenderer {
    fn render(&mut self) -> impl Future<Output = std::result::Result<(), wgpu::SurfaceError>>;
    fn render_blocking(&mut self) -> std::result::Result<(), wgpu::SurfaceError>;
}

#[derive(Default)]
pub struct RenderSubject {
    observers: HashMap<RenderEvent, Vec<Box<dyn IObserver<RenderEvent>>>>,
}

impl ISubject for RenderSubject {
    type E = RenderEvent;
    fn attach(&mut self, event: Self::E, observer: impl IObserver<Self::E> + 'static) {
        let obs = self.observers.entry(event).or_default();
        obs.push(Box::new(observer));
    }

    fn notify(&self, event: Self::E) -> void_core::Result<(), FutError<Self::E>> {
        let obs = self.observers.get(&event).unwrap();
        for o in obs {
            o.update(event)?;
        }
        Ok(())
    }
    fn detach(&mut self, _event: Self::E, _observer: impl IObserver<Self::E> + 'static) {}
}

pub struct RendererBuilder<B, T>
where
    B: IBuilder<Output = T>,
    T: IRenderer,
{
    builder: B,
}

pub struct WindowResource<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub window: Arc<Window>,
}

impl<'a> WindowResource<'a> {
    pub async fn new(window: Arc<Window>) -> Arc<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Arc::new(Self {
            adapter,
            surface,
            device,
            queue,
            config,
            window,
        })
    }
}
