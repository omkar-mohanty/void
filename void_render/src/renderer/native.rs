use crate::gui::GuiRenderer;
use egui::Context;
use egui_wgpu::wgpu;
use egui_winit::winit::window::Window;
use void_core::{Event, EventListner, Result, System};
use void_native::{MpscReceiver, MpscSender, NativeEvent, NativeEventReceiver};

use crate::{RenderEngine, RenderEvent};

impl<T: Event + 'static> System
    for RenderEngine<MpscSender<RenderEvent>, MpscReceiver<T>, RenderEvent, T>
{
    type Sender = MpscSender<RenderEvent>;
    type Receiver = MpscReceiver<T>;
    type EventUp = RenderEvent;
    type EventDown = T;

    async fn run(&mut self, func: impl FnOnce(Self::EventDown) -> Self::EventUp) -> Result<()> {
        let event = self.event_receiver.receieve_event().await?;
        let render_event = func(event);
        self.handle_render_event(render_event);

        Ok(())
    }
}

impl RenderEngine<MpscSender<RenderEvent>, NativeEventReceiver, RenderEvent, NativeEvent> {
    pub async fn new(
        context: Context,
        window: &Window,
        event_receiver: NativeEventReceiver,
        event_sender: MpscSender<RenderEvent>,
    ) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(&window).unwrap();

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

        Self {
            config,
            context,
            gui_renderer,
            device,
            queue,
            event_sender,
            event_receiver,
        }
    }
}
