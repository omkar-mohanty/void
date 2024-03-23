use crate::{gui::GuiRenderer, RenderCmd};
use egui::Context;
use egui_winit::winit::dpi::PhysicalSize;
use void_core::{CmdReceiver, Result, Subject, System};
use void_native::MpscReceiver;

use crate::{RenderEngine, RenderEvent};

impl<'a, P> System for RenderEngine<'a, P>
where
    P: Subject<E = RenderEvent>,
{
    type R = MpscReceiver<RenderCmd>;
    type C = RenderCmd;

    async fn run(&mut self, mut receiver: Self::R) -> Result<()> {
        loop {
            if let Some(cmd) = receiver.recv().await {
                self.handle_render_cmd(cmd);
            }
        }
    }
}

impl<'a, P> RenderEngine<'a, P>
where
    P: Subject<E = RenderEvent>,
{
    pub async fn new(
        context: Context,
        size: PhysicalSize<u32>,
        surface: wgpu::Surface<'a>,
        publisher: P,
        adapter: wgpu::Adapter,
    ) -> Self {
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

        Self {
            config,
            context,
            gui_renderer,
            device,
            queue,
            publisher,
            surface,
        }
    }
}
