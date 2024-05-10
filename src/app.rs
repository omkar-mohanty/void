use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    vec,
};

use crate::{
    camera::{CameraController, StaticCamera},
    gpu::Gpu,
    io::{GuiRenderer, IoEngine, Ui},
    model, resource, texture, ModelEntry, Renderer, Resources,
};
use egui::{Align2, Context};
use transform_gizmo_egui::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::Window,
};

pub struct App {
    resources: Arc<Resources>,
    gpu: Arc<Gpu>,
    renderer: Renderer,
    io_engine: IoEngine<Arc<RwLock<CameraController>>>,
}

#[derive(Default)]
struct Gui {
    gizmo: Gizmo,
    camera: Arc<RwLock<StaticCamera>>,
}

impl Gui {
    pub fn new(camera: Arc<RwLock<StaticCamera>>) -> Self {
        let gizmo = Gizmo::default();
        Self { gizmo, camera }
    }
}

impl Ui for Gui {
    fn render_ui(&mut self, ctx: &Context) {
        egui::Window::new("Control Plane")
            .default_open(true)
            .resizable(true)
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(ctx, |ui| {
                if ui.add(egui::Button::new("Clicked")).clicked() {
                    println!("Clicked");
                }
            });
    }
}

impl App {
    pub async fn new(window: Window) -> Self {
        let window = Arc::new(window);

        let gpu = Arc::new(Gpu::new(Arc::clone(&window)).await);
        let gui = Gui::default();

        let controller = Arc::new(RwLock::new(CameraController::default()));
        let camera = Arc::new(RwLock::new(StaticCamera::new()));

        let renderer = Renderer::new(
            Arc::clone(&window),
            Arc::clone(&gpu),
            Arc::clone(&controller),
            Arc::clone(&camera),
        )
        .await;

        let resources = Arc::new(Resources::new());
        let gui_renderer = GuiRenderer::new(Arc::clone(&gpu), None, 1, Arc::clone(&window), gui);
        let io_engine = IoEngine::new(
            Arc::clone(&gpu),
            Arc::clone(&resources),
            Arc::clone(&window),
            gui_renderer,
            controller,
        );

        Self {
            renderer,
            resources,
            gpu,
            io_engine,
        }
    }

    pub async fn handle_file_drop(&mut self, path: &PathBuf) -> anyhow::Result<()> {
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

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
    pub async fn run(&mut self, event_loop: EventLoop<()>) {
        let _ = event_loop.run(move |event, ewlt| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.renderer.window().id() => {
                if !self.renderer.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key: Key::Named(NamedKey::Escape),
                                    ..
                                },
                            ..
                        } => ewlt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            log::info!("Resized");
                            self.renderer.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            log::info!("Redraw");

                            self.renderer.update();

                            let model_read = self.resources.model_db.read().unwrap();
                            let models = model_read.get_all();

                            match self.renderer.render_models(models) {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    self.renderer.resize(self.renderer.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => ewlt.exit(),
                                // We're ignoring timeouts
                                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                            };
                            self.io_engine.render();
                            self.gpu.finish();
                        }
                        WindowEvent::DroppedFile(path) => {
                            match futures::executor::block_on(self.handle_file_drop(path)) {
                                Ok(()) => log::info!("Model Added"),
                                Err(err) => log::error!("Model Could not be added : {err}"),
                            }
                        }
                        _ => {
                            log::info!("Other");
                        }
                    };
                    self.io_engine.handle_event(event);
                }
            }
            _ => {}
        });
    }
}
