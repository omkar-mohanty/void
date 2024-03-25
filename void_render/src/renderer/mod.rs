use core::fmt;
use std::{fmt::Write, iter, sync::Arc};

use crate::gui::GuiRenderer;
use egui::FullOutput;
use void_core::{CmdReceiver, Command, Event, Result, Subject};
use winit::window::Window;

mod model;
pub mod pipeline;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[derive(Clone)]
pub enum RenderCmd {
    GuiOutput(FullOutput),
    Render,
    Resize { height: u32, width: u32 },
}

impl fmt::Display for RenderCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RenderCmd::*;
        match self {
            GuiOutput(_) => f.write_str("GuiOutput"),
            Render => f.write_str("Render"),
            Resize { .. } => f.write_str("Resize"),
        }
    }
}

impl Command for RenderCmd {}

pub enum RenderEvent {
    PassComplete,
}

impl Event for RenderEvent {}

pub struct RenderEngine<'a, P, R>
where
    P: Subject<E = RenderEvent>,
    R: CmdReceiver<RenderCmd>,
{
    window: Arc<Window>,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    gui_renderer: GuiRenderer,
    subject: P,
    receiver: R,
    full_output: Option<FullOutput>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl<'a, P, R> RenderEngine<'a, P, R>
where
    P: Subject<E = RenderEvent>,
    R: CmdReceiver<RenderCmd>,
{
    fn update_gui(
        &mut self,
        full_output: FullOutput,
        encoder: &mut wgpu::CommandEncoder,
        texture_view: &wgpu::TextureView,
        pixels_per_point: f32,
    ) {
        let size = self.window.inner_size();
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point,
        };
        self.gui_renderer.update(
            &self.device,
            &self.queue,
            encoder,
            &texture_view,
            screen_descriptor,
            full_output,
        );
    }

    fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
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

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw(0..3, 0..1);
        }

        if self.full_output.is_some() {
            let full_output = self.full_output.take().unwrap();
            self.update_gui(
                full_output,
                &mut encoder,
                &view,
                self.window.scale_factor() as f32,
            );
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
    }

    fn handle_cmd(&mut self, render_event: RenderCmd) {
        use RenderCmd::*;
        match render_event {
            Resize { height, width } => {
                self.config.height = height;
                self.config.width = width;
            }
            GuiOutput(full_output) => self.full_output = Some(full_output),
            Render => self.render(),
        }
        self.subject.notify(RenderEvent::PassComplete);
        log::info!("Render Notified");
    }
}
