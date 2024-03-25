use egui::epaint::Shadow;
use egui::{Context, Visuals};
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;
use egui_winit::State;
use std::ops::Deref;
use std::sync::Arc;
use void_core::Result;

use crate::{IBuilder, IRenderer, RendererBuilder, WindowResource};

#[derive(Default)]
struct GuiRendererBuilder<'a> {
    msaa_samples: Option<u32>,
    egui_context: Option<Context>,
    resource: Option<Arc<WindowResource<'a>>>,
}

impl<'a> IBuilder for GuiRendererBuilder<'a> {
    type Output = GuiRenderer<'a>;

    async fn build(self) -> Result<Self::Output> {
        let GuiRendererBuilder {
            msaa_samples,
            egui_context,
            resource,
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

        Ok(GuiRenderer::new(msaa_samples, egui_context, resource).await)
    }
}

impl<'a> RendererBuilder<GuiRendererBuilder<'a>, GuiRenderer<'a>> {
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

impl<'a> IBuilder for RendererBuilder<GuiRendererBuilder<'a>, GuiRenderer<'a>> {
    type Output = GuiRenderer<'a>;

    async fn build(self) -> Result<Self::Output> {
        self.builder.build().await
    }
}

impl IRenderer for GuiRenderer<'_> {
    async fn render(&mut self) {
        todo!("Implement Gui Render Async");
    }
    fn render_blocking(&mut self) {
        todo!("Implement Gui Render Async");
    }
}

pub struct GuiRenderer<'a> {
    resource: Arc<WindowResource<'a>>,
    state: State,
    context: Context,
    renderer: Renderer,
}

impl<'a> GuiRenderer<'a> {
    pub async fn new(
        msaa_samples: u32,
        egui_context: Context,
        resource: Arc<WindowResource<'a>>,
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

        Self {
            resource,
            state,
            context: egui_context,
            renderer: egui_renderer,
        }
    }

    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        window_surface_view: &wgpu::TextureView,
        screen_descriptor: ScreenDescriptor,
        run_ui: impl FnOnce(&Context),
    ) {
        let WindowResource {
            device,
            queue,
            window,
            ..
        } = self.resource.deref();

        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, |ui| {
            run_ui(&self.context);
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
