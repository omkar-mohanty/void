use egui::epaint::Shadow;
use egui::{Context, FullOutput, RawInput, Visuals};
use egui_wgpu::wgpu;
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;

use wgpu::{Device, TextureFormat};

use void_core::{Event, System};

use crate::Gui;

pub struct GuiEvent<'a> {
    gui: &'a mut dyn Gui,
    raw_input: RawInput,
}

impl Event for GuiEvent<'_> {}

impl System for GuiRenderer {
    type T = GuiEvent<'static>;
    type S = ();
    type R = FullOutput;

    fn process_event(&mut self, event: GuiEvent) -> FullOutput {
        let GuiEvent { gui, raw_input } = event;
        self.draw(raw_input, gui)
    }
}

pub struct GuiRenderer {
    context: Context,
    renderer: Renderer,
}

impl GuiRenderer {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        egui_context: Context,
    ) -> Self {
        const BORDER_RADIUS: f32 = 2.0;

        let visuals = Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: Shadow::NONE,
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_renderer = egui_wgpu::Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
        );

        Self {
            context: egui_context,
            renderer: egui_renderer,
        }
    }

    pub fn update_inner(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        texture_view: &wgpu::TextureView,
        screen_descriptor: ScreenDescriptor,
        full_output: egui::FullOutput,
    ) {
        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&device, &queue, *id, &image_delta);
        }
        self.renderer
            .update_buffers(&device, &queue, encoder, &tris, &screen_descriptor);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
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

    fn draw(&mut self, raw_input: egui::RawInput, gui: &mut dyn Gui) -> egui::FullOutput {
        self.context.run(raw_input, |_ui| {
            gui.run(&self.context);
        })
    }
}
