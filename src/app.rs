use std::{path::PathBuf, sync::Arc, vec};

use crate::{gpu::Gpu, model, resource, texture, ModelEntry, Renderer, Resources};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

pub struct App {
    resources: Arc<Resources>,
    gpu: Arc<Gpu>,
    renderer: Renderer,
    event_loop: EventLoop<()>,
}

impl App {
    pub async fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

        let gpu = Arc::new(Gpu::new(Arc::clone(&window)).await);

        let renderer = Renderer::new(window, Arc::clone(&gpu)).await;
        let resources = Arc::new(Resources::new());

        Self {
            event_loop,
            renderer,
            resources,
            gpu,
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
    pub async fn run(mut self) {
        let renderer = &mut self.renderer;

        let _ = self.event_loop.run(move |event, ewlt| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == renderer.window().id() => {
                if !renderer.input(event) {
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
                            renderer.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            log::info!("Redraw");
                            renderer.update();
                            match renderer.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    renderer.resize(renderer.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => ewlt.exit(),
                                // We're ignoring timeouts
                                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                            }
                        }
                        _ => {
                            log::info!("Other");
                        }
                    };
                    renderer.egui.handle_input(&mut renderer.window, &event);
                }
            }
            _ => {}
        });
    }
}
