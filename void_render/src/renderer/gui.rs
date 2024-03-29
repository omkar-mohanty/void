use crate::{Draw, IRenderer, RendererBuilder};
use egui::epaint::Shadow;
use egui::{Context as GuiContext, Visuals};
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;
use egui_winit::State;
use std::ops::Deref;
use std::sync::Arc;
use void_core::{BuilderError, IBuilder, IGui, Result};
use void_gpu::GpuResource;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

#[derive(Default)]
pub struct GuiRendererBuilder<'a, T: IGui + Default + Send> {
    msaa_samples: Option<u32>,
    egui_context: Option<GuiContext>,
    resource: Option<Arc<GpuResource<'a, Window>>>,
    gui: Option<T>,
}

impl<'a, T: IGui + Default + Send> IBuilder for GuiRendererBuilder<'a, T> {
    type Output = GuiRenderer<'a, T>;

    async fn build(self) -> Result<Self::Output, BuilderError> {
        let GuiRendererBuilder {
            msaa_samples,
            egui_context,
            resource,
            gui,
        } = self;

        let msaa_samples = msaa_samples.unwrap_or(2);

        let egui_context = match egui_context {
            Some(context) => context,
            None => todo!(),
        };

        let resource = match resource {
            Some(resource) => resource,
            None => todo!(),
        };

        let gui = match gui {
            Some(gui) => gui,
            None => todo!(),
        };

        Ok(GuiRenderer::new(msaa_samples, egui_context, resource, gui).await)
    }
}

impl<'a, T: IGui + Default + Send> RendererBuilder<GuiRendererBuilder<'a, T>, GuiRenderer<'a, T>> {
    fn new_gui() -> Self {
        Self {
            builder: GuiRendererBuilder::default(),
        }
    }

    pub fn set_msaa(mut self, samples: u32) -> Self {
        self.builder.msaa_samples = Some(samples);
        self
    }

    pub fn set_context(mut self, context: GuiContext) -> Self {
        self.builder.egui_context = Some(context);
        self
    }

    pub fn set_resource(mut self, resource: Arc<GpuResource<'a, Window>>) -> Self {
        self.builder.resource = Some(resource);
        self
    }

    pub fn set_gui(mut self, gui: T) -> Self {
        self.builder.gui = Some(gui);
        self
    }
}

impl<'a, T: IGui + Default + Send> IBuilder
    for RendererBuilder<GuiRendererBuilder<'a, T>, GuiRenderer<'a, T>>
{
    type Output = GuiRenderer<'a, T>;

    async fn build(self) -> Result<Self::Output, BuilderError> {
        let res = self.builder.build().await?;
        Ok(res)
    }
}

impl<T: IGui + Send + Default> IRenderer for GuiRenderer<'_, T> {
    async fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        self.render_blocking()
    }

    fn render_blocking(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        let output = self.resource.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let mut encoder =
            self.resource
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Gui Renderer Command Encoder"),
                });

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.resource.window.scale_factor() as f32,
        };
        self.draw_ui(&mut encoder, &view, screen_descriptor);
        self.resource
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

pub struct GuiRenderer<'a, T: IGui> {
    resource: Arc<GpuResource<'a, Window>>,
    state: State,
    context: GuiContext,
    renderer: Renderer,
    config: wgpu::SurfaceConfiguration,
    gui: T,
}

impl<'a, T: IGui + Send + Default> GuiRenderer<'a, T> {
    pub fn builder() -> RendererBuilder<GuiRendererBuilder<'a, T>, Self> {
        RendererBuilder::new_gui()
    }

    pub async fn new(
        msaa_samples: u32,
        egui_context: GuiContext,
        resource: Arc<GpuResource<'a, Window>>,
        gui: T,
    ) -> Self {
        const BORDER_RADIUS: f32 = 2.0;
        let GpuResource {
            device,
            config,
            window,
            ..
        } = resource.deref();
        let config = config.clone();

        let color_format = config.format;

        let viewport_id = egui_context.viewport_id();

        let visuals = Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: Shadow::NONE,
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_renderer = egui_wgpu::Renderer::new(&device, color_format, None, msaa_samples);

        let state = egui_winit::State::new(egui_context.clone(), viewport_id, &window, None, None);

        Self {
            config,
            resource,
            state,
            context: egui_context,
            renderer: egui_renderer,
            gui,
        }
    }

    pub fn draw_ui(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        window_surface_view: &wgpu::TextureView,
        screen_descriptor: ScreenDescriptor,
    ) {
        let GpuResource {
            device,
            queue,
            window,
            ..
        } = self.resource.deref();

        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, |_ui| {
            self.gui.show(&self.context);
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, &image_delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            label: Some("egui main render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.resource
                .surface
                .configure(&self.resource.device, &self.config);
        }
    }

    pub fn gather_input(&mut self, event: &WindowEvent) {
        let _ = self.state.on_window_event(&self.resource.window, event);
    }

    #[allow(unused_variables)]
    pub fn input(&self, event: &WindowEvent) -> bool {
        self.resource.window.request_redraw();
        false
    }
}

impl<'a, T: IGui + Send + Default> Draw for GuiRenderer<'a, T> {
    fn draw(&mut self, view: Arc<wgpu::TextureView>) {
        let mut encoder =
            self.resource
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Gui Command Encoder"),
                });
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.resource.window.scale_factor() as f32,
        };
        self.draw_ui(&mut encoder, &view, screen_descriptor);
        self.resource
            .queue
            .submit(std::iter::once(encoder.finish()));
    }
}
