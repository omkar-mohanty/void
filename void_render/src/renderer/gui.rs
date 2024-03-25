use egui::epaint::Shadow;
use egui::{Context, Visuals};
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;
use egui_winit::State;
use winit::dpi::PhysicalSize;
use std::ops::Deref;
use std::sync::Arc;
use void_core::{IGui, Result};

use crate::{renderer, IBuilder, IRenderer, RendererBuilder, WindowResource};

#[derive(Default)]
struct GuiRendererBuilder<'a, T: IGui + Default + Send> {
    msaa_samples: Option<u32>,
    egui_context: Option<Context>,
    resource: Option<Arc<WindowResource<'a>>>,
    gui: Option<T>,
}

impl<'a, T: IGui + Default + Send> IBuilder for GuiRendererBuilder<'a, T> {
    type Output = GuiRenderer<'a, T>;

    async fn build(self) -> Result<Self::Output> {
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
    pub fn new() -> Self {
        Self {
            builder: GuiRendererBuilder::default(),
        }
    }

    pub fn set_msaa(mut self, samples: u32) -> Self {
        self.builder.msaa_samples = Some(samples);
        self
    }

    pub fn set_context(mut self, context: Context) -> Self {
        self.builder.egui_context = Some(context);
        self
    }

    pub fn set_resource(mut self, resource: Arc<WindowResource<'a>>) -> Self {
        self.builder.resource = Some(resource);
        self
    }
}

impl<'a, T: IGui + Default + Send> IBuilder for RendererBuilder<GuiRendererBuilder<'a, T>, GuiRenderer<'a, T>> {
    type Output = GuiRenderer<'a, T>;

    async fn build(self) -> Result<Self::Output> {
        self.builder.build().await
    }
}

impl<T: IGui> IRenderer for GuiRenderer<'_, T> {
    async fn render(&mut self) {
        todo!("Implement Gui Render Async");
    }
    fn render_blocking(&mut self) {
        todo!("Implement Gui Render Async");
    }
}

pub struct GuiRenderer<'a, T: IGui> {
    resource: Arc<WindowResource<'a>>,
    state: State,
    context: Context,
    renderer: Renderer,
    config: wgpu::SurfaceConfiguration,
    gui: T,
}

impl<'a, T: IGui> GuiRenderer<'a, T> {
    pub async fn new(
        msaa_samples: u32,
        egui_context: Context,
        resource: Arc<WindowResource<'a>>,
        gui:T,
    ) -> Self {
        const BORDER_RADIUS: f32 = 2.0;
        let WindowResource {
            device,
            config,
            window,
            ..
        } = resource.deref();

        let output_color_format = config.format;

        let viewport_id = egui_context.viewport_id();

        let visuals = Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: Shadow::NONE,
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_renderer =
            egui_wgpu::Renderer::new(&device, output_color_format, None, msaa_samples);

        let state = egui_winit::State::new(egui_context.clone(), viewport_id, &window, None, None);

        let config = resource.config.clone();

        Self {
            resource,
            state,
            context: egui_context,
            renderer: egui_renderer,
            gui,
            config
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
    }

    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        window_surface_view: &wgpu::TextureView,
        screen_descriptor: ScreenDescriptor,
    ) {
        let WindowResource {
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
}
