use std::sync::Arc;

use crate::{gpu::Gpu, resource, Renderer, Resources};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

pub struct App {
    resources: Arc<Resources>,
    renderer: Renderer,
    event_loop: EventLoop<()>,
}

impl App {
    pub async fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = Arc::new(unsafe { instance.create_surface(&window) }.unwrap());

        let gpu = Gpu::new(&instance, Arc::clone(&surface));

        let renderer = Renderer::new(window).await;
        let resources = Arc::new(Resources::new());

        Self {
            event_loop,
            renderer,
            resources,
        }
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
                            renderer.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
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
                        WindowEvent::DroppedFile(path) => {
                            todo!("Handle File drop")
                        }
                        _ => {}
                    };
                    renderer.egui.handle_input(&mut renderer.window, &event);
                }
            }
            _ => {}
        });
    }
}
