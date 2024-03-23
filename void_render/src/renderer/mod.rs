use crate::gui::GuiRenderer;
use egui::FullOutput;
use void_core::{Command, Event, Subject};

#[cfg(not(target_arch = "wasm32"))]
mod native;

pub enum RenderCmd {
    GuiOutput(FullOutput),
    Render(f32),
    Resize { height: u32, width: u32 },
}

impl Command for RenderCmd {}

pub enum RenderEvent {
    PassComplete,
}

impl Event for RenderEvent {}

pub struct RenderEngine<'a, P>
where
    P: Subject<E = RenderEvent>,
{
    surface: wgpu::Surface<'a>,
    context: egui::Context,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    gui_renderer: GuiRenderer,
    publisher: P,
}

impl<'a, P> RenderEngine<'a, P>
where
    P: Subject<E = RenderEvent>,
{
    fn update(
        &mut self,
        full_output: FullOutput,
        encoder: &mut wgpu::CommandEncoder,
        pixels_per_point: f32,
    ) {
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
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point,
        };
        self.gui_renderer.update(
            &self.device,
            &self.queue,
            encoder,
            &view,
            screen_descriptor,
            full_output,
        );
    }

    fn render(&mut self) {
        todo!()
    }

    fn handle_render_cmd(&mut self, render_event: RenderCmd) {
        use RenderCmd::*;
        match render_event {
            Resize { height, width } => {
                self.config.height = height;
                self.config.width = width;
            }
            GuiOutput(_full_output) => todo!(),
            Render(_pixels_per_point) => self.render(),
        }
    }
}
