use std::{
    cell::OnceCell,
    collections::BTreeMap,
    sync::{atomic::AtomicUsize, Arc, OnceLock, RwLock},
};

use wgpu::TextureView;

static CMD_ID: OnceLock<AtomicUsize> = OnceLock::new();

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Arc<wgpu::Surface>,
    cmds: RwLock<BTreeMap<usize, wgpu::CommandBuffer>>,
    current_texture_view: RwLock<OnceCell<wgpu::SurfaceTexture>>,
}

impl Gpu {
    pub async fn new(instance: &wgpu::Instance, surface: Arc<wgpu::Surface>) -> Self {
        CMD_ID.get_or_init(|| AtomicUsize::new(0));
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();
        Self {
            device,
            queue,
            surface,
            cmds: RwLock::new(BTreeMap::default()),
            current_texture_view: RwLock::new(OnceCell::new()),
        }
    }

    pub fn create_cmd_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Wgpu command encoder"),
            })
    }

    pub fn get_current_view(&self) -> TextureView {
        let surface_tex = self.current_texture_view.read().unwrap();
        let surface_tex = surface_tex.get_or_init(|| self.surface.get_current_texture().unwrap());
        surface_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn submit_cmd(&self, cmd: wgpu::CommandBuffer) {
        let id = CMD_ID
            .get()
            .unwrap()
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut cmds_write = self.cmds.write().unwrap();
        cmds_write.insert(id, cmd);
    }

    pub fn finish(&self) {
        let mut cmds_write = self.cmds.write().unwrap();
        let cmds = std::mem::take(&mut *cmds_write);
        let cmds = cmds.into_values().into_iter();
        self.queue.submit(cmds);
        let mut current_surface_tex = self.current_texture_view.write().unwrap();
        let current_surface_tex = current_surface_tex.take().unwrap();
        current_surface_tex.present();
    }
}
