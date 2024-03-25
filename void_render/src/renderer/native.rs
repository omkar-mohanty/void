use std::sync::Arc;
use crate::{gui::GuiRenderer, RenderCmd, renderer::model::{VERTICES, INDICES}};
use crate::{RenderEngine, RenderEvent};
use egui::Context;
use void_core::{CmdReceiver, Result, Subject, System};
use winit::window::Window;
use super::model::Vertex;
use wgpu::util::DeviceExt;

impl<'a, P, R> System for RenderEngine<'a, P, R>
where
    P: Subject<E = RenderEvent> + Send,
    R: CmdReceiver<RenderCmd>,
{
    type C = RenderCmd;

    async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(cmd) = self.receiver.recv().await {
                log::info!("Render Engine Received : {cmd}");
                self.handle_cmd(cmd);
            }
        }
    }

    fn run_blocking(&mut self) -> Result<()> {
        if let Some(cmd) = self.receiver.recv_blockding() {
            log::info!("Render Engine Received : {cmd}");
            self.handle_cmd(cmd);
        }
        Ok(())
    }
}

impl<'a, P, R> RenderEngine<'a, P, R>
where
    P: Subject<E = RenderEvent>,
    R: CmdReceiver<RenderCmd>,
{
    pub async fn new(window: Arc<Window>, context: Context, subject: P, receiver: R) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

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

        let gui_renderer = GuiRenderer::new(&device, config.format, None, 1, context.clone());

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        };

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let pipeline = crate::pipeline::create_render_pipeline(
            &device,
            &render_pipeline_layout,
            config.format,
            None,
            &[Vertex::desc()],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            window,
            config,
            gui_renderer,
            device,
            queue,
            subject,
            surface,
            receiver,
            pipeline,
            full_output: None,
        }
    }
}
