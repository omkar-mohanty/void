use std::{
    cell::OnceCell,
    collections::BTreeMap,
    sync::{atomic::AtomicUsize, Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use wgpu::TextureView;
use winit::window::Window;

static CMD_ID: OnceLock<AtomicUsize> = OnceLock::new();

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Arc<wgpu::Surface>,
    config: Arc<RwLock<wgpu::SurfaceConfiguration>>,
    window: Arc<Window>,
    current_texture_view: RwLock<OnceCell<wgpu::SurfaceTexture>>,
    cmds: RwLock<BTreeMap<usize, wgpu::CommandBuffer>>,
}

impl Gpu {
    pub async fn new(window: Arc<Window>) -> Self {
        CMD_ID.get_or_init(|| AtomicUsize::new(0));

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = Arc::new(unsafe { instance.create_surface(&window) }.unwrap());

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

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let config = Arc::new(RwLock::new(config));

        Self {
            device,
            queue,
            surface,
            cmds: RwLock::new(BTreeMap::default()),
            current_texture_view: RwLock::new(OnceCell::new()),
            window,
            config,
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

    pub fn get_config(&self) -> RwLockReadGuard<wgpu::SurfaceConfiguration> {
        self.config.read().unwrap()
    }

    pub fn get_config_mut(&self) -> RwLockWriteGuard<wgpu::SurfaceConfiguration> {
        self.config.write().unwrap()
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
