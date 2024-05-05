use egui::epaint::Shadow;
use egui::{Context, Visuals};
use egui_wgpu::renderer::ScreenDescriptor;
use egui_wgpu::Renderer;

use crate::gpu::Gpu;
use crate::model;
use crate::resource;
use crate::texture;
use crate::ModelEntry;
use crate::Resources;

use wgpu::util::DeviceExt;

use egui_winit::State;
use std::path::PathBuf;
use std::sync::Arc;
use wgpu::TextureFormat;
use winit::event::{KeyEvent, WindowEvent};
use winit::window::Window;

pub trait Controller {
    fn process_events(&self, ctx: &KeyEvent);
}

pub trait Ui {
    fn render_ui(&mut self, context: &Context);
}

pub struct IoEngine<T: Controller> {
    camera_controller: T,
    resources: Arc<Resources>,
    window: Arc<Window>,
    gui: GuiRenderer,
    gpu: Arc<Gpu>,
}

impl<T: Controller> IoEngine<T> {
    pub fn new(
        gpu: Arc<Gpu>,
        resources: Arc<Resources>,
        window: Arc<Window>,
        gui: GuiRenderer,
        camera_controller: T,
    ) -> Self {
        Self {
            camera_controller,
            resources,
            gui,
            gpu,
            window,
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        use WindowEvent::*;
        match event {
            DroppedFile(path) => match futures::executor::block_on(self.add_model(path)) {
                Ok(()) => log::info!("Added Model"),
                Err(msg) => log::error!("{msg}"),
            },
            KeyboardInput { event, .. } => {
                self.camera_controller.process_events(&event);
            }
            RedrawRequested => {
                self.gui.render_ui();
            }
            _ => {}
        };

        self.gui.handle_input(&self.window, event);
    }

    pub async fn add_model(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        let path_string = path.display().to_string();
        let layout = texture::Texture::get_bind_group_layout(&self.gpu);
        let model = resource::load_model(&path_string, &self.gpu, &layout).await?;
        let mut model_db = self.resources.model_db.write().unwrap();
        let device = &self.gpu.device;
        let instances = vec![model::Instance::default()];

        let instance_data = instances
            .iter()
            .map(model::Instance::to_raw)
            .collect::<Vec<_>>();
        let model_entry = ModelEntry {
            instances,
            instance_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model instance"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            model,
        };
        model_db.insert(model_entry);
        Ok(())
    }
}

pub struct GuiRenderer {
    context: Context,
    gpu: Arc<Gpu>,
    state: State,
    renderer: Renderer,
    window: Arc<Window>,
    ui: Box<dyn Ui>,
}

impl GuiRenderer {
    pub fn new(
        gpu: Arc<Gpu>,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: Arc<Window>,
        ui: impl Ui + 'static,
    ) -> Self {
        let device = &gpu.device;
        let output_color_format = gpu.get_config_read(|config| config.format);
        let egui_context = Context::default();
        let id = egui_context.viewport_id();

        const BORDER_RADIUS: f32 = 2.0;

        let visuals = Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: Shadow::NONE,
            // menu_rounding: todo!(),
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_state = egui_winit::State::new(egui_context.clone(), id, &window, None, None);

        // egui_state.set_pixels_per_point(window.scale_factor() as f32);
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            output_color_format,
            output_depth_format,
            msaa_samples,
        );

        GuiRenderer {
            context: egui_context,
            state: egui_state,
            renderer: egui_renderer,
            ui: Box::new(ui),
            gpu,
            window,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn render_ui(&mut self) {
        let window = &self.window;
        let mut encoder = self.gpu.create_cmd_encoder();
        let config = self.gpu.get_config();
        let window_surface_view = self.gpu.get_current_view();
        let raw_input = self.state.take_egui_input(&window);
        let full_output = self
            .context
            .run(raw_input, |_ui| self.ui.render_ui(&self.context));

        self.state
            .handle_platform_output(&window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&self.gpu.device, &self.gpu.queue, *id, &image_delta);
        }
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [config.width, config.height],
            pixels_per_point: window.scale_factor() as f32,
        };
        self.renderer.update_buffers(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );
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
        self.gpu.submit_cmd(encoder.finish());
    }
}
