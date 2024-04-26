use render_ctx::{DrawCmd, RenderCtx};
use void_core::rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use wgpu::util::{DeviceExt, RenderEncoder};

use super::{
    pipeline::{default_render_pipeline, CAMERA_BIND_GROUP_LAYOUT},
    texture::{Texture, TextureError},
    IDisplayable,
};
use crate::{
    api::{BindGroupId, BufferId, CommandListIndex, ICtxOut, IGpu, PipelineId},
    camera::CameraUniform,
};
use rand::Rng;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
    str::FromStr,
    sync::{Arc, OnceLock, RwLock},
};
use thiserror::Error;
use uuid::{uuid, Uuid};

pub mod render_ctx;
pub mod upload_ctx;

pub(crate) static DEFAULT_PIPELINE_ID: PipelineId =
    PipelineId(uuid!("36408a0a-57d7-40af-b476-ab1aa5a77ac7"));

pub(crate) static DEFAULT_CAMERA_BUFFER_ID: BufferId =
    BufferId(uuid!("059089eb-c098-4fc5-b67a-cd24db18f6f6"));

pub(crate) const DEFAULT_CAMERA_BIND_GROUP_ID: BindGroupId =
    BindGroupId(uuid!("79e5bf90-8a9d-4b34-8e04-e7f619c2c7c4"));

static CONTEXTS: OnceLock<ContextManager> = OnceLock::new();

pub(crate) static GPU_RESOURCE: OnceLock<GpuResource> = OnceLock::new();

#[derive(Default)]
pub struct RenderCmd {
    pub(crate) bind_groups_id: Vec<(u32, BindGroupId)>,
    pub(crate) vertex_buffer: Option<(u32, BufferId)>,
    pub(crate) index_buffer: Option<BufferId>,
    pub(crate) pipeline: Option<PipelineId>,
    pub(crate) draw_cmd: Option<DrawCmd>,
}

#[derive(Default)]
pub struct BufferManager {
    buffers: HashMap<BufferId, wgpu::Buffer>,
}

impl BufferManager {
    pub(crate) fn add_buffer(contents: wgpu::Buffer) -> BufferId {
        ContextManager::record(move |ctx_manager| {
            let mut buffer_mgr = ctx_manager.buffer_manager.write().unwrap();
            let id = BufferId(Uuid::new_v4());
            buffer_mgr.buffers.insert(id, contents);
            id
        })
    }
}

#[derive(Default)]
pub struct ContextManager {
    pub(crate) render_ctxs: RwLock<HashMap<Uuid, RenderCmd>>,
    pub(crate) buffer_manager: RwLock<BufferManager>,
    pub(crate) pipeline_db: PipelineDB,
}

impl ContextManager {
    fn init() {
        let contexts = CONTEXTS.get_or_init(|| ContextManager::default());
        let mut buf_mgr = contexts.buffer_manager.write().unwrap();

        let resource = GPU_RESOURCE.get().unwrap();
        let default_pipeline =
            default_render_pipeline(&resource.device, &resource.config.read().unwrap());
        let mut pipelines_db = contexts.pipeline_db.pipelines.write().unwrap();

        let camera_buffer = resource
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(""),
                contents: bytemuck::cast_slice(&[CameraUniform::new()]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let mut bind_groups = HashMap::new();

        bind_groups.insert(
            DEFAULT_CAMERA_BIND_GROUP_ID,
            resource
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Camera Bind Group"),
                    layout: &CAMERA_BIND_GROUP_LAYOUT.get().unwrap(),
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }],
                }),
        );

        buf_mgr
            .buffers
            .insert(DEFAULT_CAMERA_BUFFER_ID, camera_buffer);

        pipelines_db.insert(
            DEFAULT_PIPELINE_ID,
            PipelineEntry {
                pipeline: default_pipeline,
                source: String::from_str(include_str!("../shader.wgsl")).unwrap(),
                bind_groups,
            },
        );
    }

    pub(crate) fn record<T, F: FnOnce(&Self) -> T>(func: F) -> T {
        if let None = CONTEXTS.get() {
            Self::init();
        }
        let ctxs = CONTEXTS.get().unwrap();
        func(ctxs)
    }
}

pub(crate) struct PipelineEntry {
    pub(crate) pipeline: GpuPipeline,
    pub(crate) source: String,
    pub(crate) bind_groups: HashMap<BindGroupId, wgpu::BindGroup>,
}

pub(crate) struct GpuResource {
    pub(crate) window: Arc<dyn IDisplayable>,
    pub(crate) surface: RwLock<wgpu::Surface<'static>>,
    pub(crate) config: RwLock<wgpu::SurfaceConfiguration>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}

impl GpuResource {
    pub async fn init<T: IDisplayable + 'static>(window: Arc<T>, width: u32, height: u32) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

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
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let mut buffers = HashMap::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::UNIFORM,
        });

        buffers.insert(DEFAULT_CAMERA_BUFFER_ID, camera_buffer);

        let res = Self {
            window,
            surface: RwLock::new(surface),
            queue,
            device,
            config: RwLock::new(config),
        };

        GPU_RESOURCE.get_or_init(|| res);
    }
}

#[derive(Default)]
pub struct PipelineDB {
    pub(crate) pipelines: RwLock<HashMap<PipelineId, PipelineEntry>>,
}

pub enum CtxOut {
    Render(RenderCtx),
}

impl ICtxOut for CtxOut {}

pub enum GpuPipeline {
    Render(wgpu::RenderPipeline),
    Compute(wgpu::ComputePipeline),
}

pub struct Gpu {
    node_id: [u8; 6],
    render_ctxs: RwLock<BTreeMap<CommandListIndex, RenderCtx>>,
    static_cmds: RwLock<BTreeMap<CommandListIndex, wgpu::RenderBundle>>,
    depth_texture: RwLock<Texture>,
}

impl Gpu {
    async fn init<T: IDisplayable + 'static>(window: Arc<T>, width: u32, height: u32) {
        GpuResource::init(window, width, height).await;
        ContextManager::init();
    }
    pub async fn new<T: IDisplayable + 'static>(
        window: Arc<T>,
        width: u32,
        height: u32,
    ) -> Arc<Self> {
        Self::init(window, width, height).await;

        let resource = GPU_RESOURCE.get().unwrap();

        let depth_texture = RwLock::new(Texture::create_depth_texture(
            &resource.device,
            &resource.config.read().unwrap(),
            "Depth Texture",
        ));

        Arc::new(Self {
            node_id: rand::thread_rng().gen::<[u8; 6]>(),
            render_ctxs: RwLock::new(BTreeMap::new()),
            static_cmds: RwLock::new(BTreeMap::new()),
            depth_texture,
        })
    }

    pub(crate) fn get_resource(&self) -> &'static GpuResource {
        GPU_RESOURCE.get().unwrap()
    }

    pub fn get_window(&self) -> &'static dyn IDisplayable {
        self.get_resource().window.deref()
    }
}

impl IGpu for Gpu {
    type Err = GpuError;
    type CtxOut = CtxOut;

    fn create_buffer(&self) -> BufferId {
        todo!()
    }

    fn window_update(&self, width: u32, height: u32) {
        if width <= 0 || height <= 0 {
            return;
        }

        let resource = GPU_RESOURCE.get().unwrap();
        let device = &resource.device;
        let mut config = resource.config.write().unwrap();
        let surface = resource.surface.read().unwrap();

        let mut depth_tex = self.depth_texture.write().unwrap();

        config.width = width;
        config.height = height;

        let _ = std::mem::replace(
            &mut *depth_tex,
            Texture::create_depth_texture(device, &config, "Depth Texture"),
        );

        surface.configure(device, &config);
    }

    fn submit_ctx_out(&self, out: Self::CtxOut) {
        match out {
            CtxOut::Render(ctx) => {
                let mut ctxs = self.render_ctxs.write().unwrap();
                ctxs.insert(CommandListIndex::new(&self.node_id), ctx);
            }
        }
    }

    fn present(&self) -> Result<(), Self::Err> {
        let cmds = ContextManager::record(|ctx| {
            let mut rencer_cmds = ctx.render_ctxs.write().unwrap();
            std::mem::take(&mut *rencer_cmds)
        });

        let mut ctxs_write = self.render_ctxs.write().unwrap();
        let ctxs = std::mem::take(&mut *ctxs_write);

        let resource = self.get_resource();
        let device = &resource.device;
        let surface = resource.surface.read().unwrap();
        let queue = &resource.queue;

        let context = &CONTEXTS.get().unwrap();
        let manager = context.buffer_manager.read().unwrap();
        let pipeline_db = &context.pipeline_db;
        let pipelines = &pipeline_db.pipelines.read().unwrap();

        let current_texture = surface.get_current_texture()?;
        let view = current_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_tex = &self.depth_texture.read().unwrap().view;

        let cmds_ordered:Vec<_> = ctxs
            .into_par_iter()
            .map(|(_, ctx)| {
                let id = ctx.id;
                cmds.get(&id).unwrap()
            })
            .map(|cmd| {
                let mut cmd_encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Command encoder"),
                    });

                {
                    let mut rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_tex,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    let RenderCmd {
                        bind_groups_id,
                        vertex_buffer,
                        index_buffer,
                        pipeline,
                        draw_cmd,
                    } = cmd;

                    let PipelineEntry {
                        pipeline,
                        bind_groups,
                        ..
                    } = pipelines
                        .get(&pipeline.unwrap_or(DEFAULT_PIPELINE_ID))
                        .unwrap();

                    for bind_group_entry in bind_groups_id {
                        let slot = bind_group_entry.0;
                        let bind_group_id = bind_group_entry.1;
                        let bind_group = bind_groups.get(&bind_group_id).unwrap();
                        rpass.set_bind_group(slot, bind_group, &[]);
                    }

                    if let Some((slot, vertex_buffer_id)) = vertex_buffer  {
                        let buffer = manager.buffers.get(vertex_buffer_id).unwrap();
                        rpass.set_vertex_buffer(*slot, buffer.slice(..));
                    }

                    if let Some(index_buffer_id) = index_buffer {
                        let buffer = manager.buffers.get(index_buffer_id).unwrap();
                        rpass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
                    }

                    if let GpuPipeline::Render(pipeline) = pipeline {
                        rpass.set_pipeline(pipeline);
                    }
                        
                    if let Some(cmd) = draw_cmd {
                        let DrawCmd { indices, base_vertex, instances } = cmd;
                        rpass.draw_indexed(indices.clone(), *base_vertex, instances.clone());
                    }

                }
                cmd_encoder.finish()
            }).collect();

        queue.submit(cmds_ordered);
        current_texture.present();
        Ok(())

    }
}

#[derive(Error, Debug)]
pub enum GpuError {
    #[error("Error creating texture {0}")]
    TextureError(#[from] TextureError),
    #[error("Error {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
}
