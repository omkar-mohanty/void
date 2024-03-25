use core::fmt;
use std::{future::Future, sync::Arc};

use void_core::{ICommand, IEvent, Result};
use winit::window::Window;

use self::model::Vertex;

pub mod gui;
pub mod model;
pub mod pipeline;
pub mod scene;

#[derive(Clone)]
pub enum RenderCmd {
    Render,
}

impl fmt::Display for RenderCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RenderCmd::*;
        match self {
            Render => f.write_str("Render"),
        }
    }
}

impl ICommand for RenderCmd {}

pub enum RenderEvent {
    PassComplete,
}

impl IEvent for RenderEvent {}

pub trait IRenderer {
    fn render(&mut self) -> impl Future<Output = ()>;
    fn render_blocking(&mut self);
}

pub trait IBuilder {
    type Output;

    fn build(self) -> impl Future<Output = Result<Self::Output>> + Send;
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
    pub pipeline: wgpu::RenderPipeline,
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
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
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

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        };

        let pipeline = pipeline::create_render_pipeline(
            &device,
            &layout,
            config.format,
            None,
            &[Vertex::desc()],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
        );

        Arc::new(Self {
            adapter,
            surface,
            device,
            queue,
            config,
            pipeline,
            window,
        })
    }
}
