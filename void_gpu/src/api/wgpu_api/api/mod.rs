use render_ctx::{DrawCmd, RenderCtx};
use wgpu::{core::instance, util::DeviceExt};

use self::upload_ctx::UploadCtx;

use super::{
    pipeline::default_render_pipeline,
    texture::{Texture, TextureError},
    IDisplayable,
};
use crate::api::{BufferId, CommandListIndex, Displayable, IBuffer, ICtxOut, IGpu, PipelineId};
use rand::Rng;
use std::{
    cell::OnceCell, collections::{BTreeMap, HashMap}, ops::Deref, sync::{Arc, OnceLock, RwLock, RwLockWriteGuard}
};
use thiserror::Error;
use uuid::{uuid, Uuid};
use void_core::rayon::iter::{IntoParallelIterator, ParallelIterator};

pub mod render_ctx;
pub mod upload_ctx;

impl IBuffer for wgpu::Buffer {}

pub(crate) static DEFAULT_CAMERA_BUFFER_ID: BufferId =
    BufferId(uuid!("5b929ea6-7547-4e70-89a0-6f9ad7f9ffe4"));

pub(crate) static SURFACE: OnceLock<wgpu::Surface> = OnceLock::new();
pub(crate) static WINDOW: OnceLock<Arc<dyn IDisplayable>> = OnceLock::new();
pub(crate) static CONTEXTS: OnceLock<ContextManager> = OnceLock::new();

#[derive(Default)]
pub struct RenderCmd<'a> {
    pub(crate) bind_groups: Vec<(u32, &'a wgpu::BindGroup, &'a [wgpu::DynamicOffset])>,
    pub(crate) vertex_buffer: Option<(u32, &'a wgpu::Buffer)>,
    pub(crate) index_buffer: Option<&'a wgpu::Buffer>,
    pub(crate) pipeline: Option<PipelineId>,
    pub(crate) draw_cmd: Option<DrawCmd>,
}

#[derive(Default)]
pub struct ContextManager<'a> {
    pub(crate) render_ctxs: RwLock<HashMap<Uuid, RenderCmd<'a>>>,
}

pub struct PipelineDB {
    pub(crate) pipelines: HashMap<PipelineId, GpuPipeline>,
    default_pipeline_id: PipelineId,
}

impl PipelineDB {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let mut pipelines = HashMap::new();
        let pipeline = default_render_pipeline(device, config);
        let default_pipeline_id = PipelineId(Uuid::new_v4());

        pipelines.insert(default_pipeline_id, pipeline);

        Self {
            pipelines,
            default_pipeline_id,
        }
    }

    pub fn get_default(&self) -> &GpuPipeline {
        self.pipelines.get(&self.default_pipeline_id).unwrap()
    }
}

pub enum CtxOut {
    Render(RenderCtx),
}

impl ICtxOut for CtxOut {}

pub enum GpuPipeline {
    Render(wgpu::RenderPipeline),
    Compute(wgpu::ComputePipeline),
}

pub struct Gpu
where
{
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: RwLock<wgpu::SurfaceConfiguration>,
    pub(crate) buffers: RwLock<HashMap<BufferId, wgpu::Buffer>>,
    pub(crate) pipeline_db: RwLock<PipelineDB>,

    node_id: [u8; 6],
    render_ctxs: RwLock<BTreeMap<CommandListIndex, RenderCtx>>,
    static_cmds: RwLock<BTreeMap<CommandListIndex, wgpu::RenderBundle>>,
    depth_texture: RwLock<Texture>,
}

impl Gpu {
    
    pub async fn new<T: IDisplayable + 'static>(window: Arc<T>, width: u32, height: u32) -> Arc<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let window = WINDOW.get_or_init(|| window);

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

        let depth_texture = RwLock::new(Texture::create_depth_texture(
            &device,
            &config,
            "Depth Texture",
        ));

        let mut buffers = HashMap::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::UNIFORM,
        });

        buffers.insert(DEFAULT_CAMERA_BUFFER_ID, camera_buffer);

        let pipeline_db = PipelineDB::new(&device, &config);

        SURFACE.get_or_init(|| surface);

        Arc::new(Self {
            device,
            queue,
            config: RwLock::new(config),
            node_id: rand::thread_rng().gen::<[u8; 6]>(),
            render_ctxs: RwLock::new(BTreeMap::new()),
            static_cmds: RwLock::new(BTreeMap::new()),
            buffers: RwLock::new(buffers),
            pipeline_db: RwLock::new(pipeline_db),
            depth_texture,
        })
    }

    fn update_buffers(&self, ctx: UploadCtx) {
        let UploadCtx {
            buffer_id, data, ..
        } = ctx;

        let buffers_read = self.buffers.read().unwrap();

        if let Some(id) = buffer_id {
            if let Some(buffer) = buffers_read.get(&id) {
                self.queue.write_buffer(buffer, 0, data.unwrap_or(&[]));
            }
        }
    }
}

impl IGpu for Gpu
{
    type Err = GpuError;
    type CtxOut = CtxOut;
    type Pipeline = GpuPipeline;

    fn create_buffer(&self) -> BufferId {
        let mut buffers = self.buffers.write().unwrap();
        let buffer_desc = wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM,
            contents: &[],
        };
        let buffer = self.device.create_buffer_init(&buffer_desc);
        let id = BufferId(Uuid::new_v4());
        buffers.insert(id, buffer);
        id
    }

    fn window_update(&self, width: u32, height: u32) {
        if width <= 0 || height <= 0 {
            return;
        }
        let mut config_write = self.config.write().unwrap();
        let mut depth_tex = self.depth_texture.write().unwrap();
        config_write.width = width;
        config_write.height = height;
        let _ = std::mem::replace(
            &mut *depth_tex,
            Texture::create_depth_texture(&self.device, &config_write, "Depth Texture"),
        );
        SURFACE.get().unwrap().configure(&self.device, &config_write);
    }

    fn submit_ctx_out(&self, out: Self::CtxOut) {
        todo!()
    }

    fn present(&self) -> Result<(), Self::Err> {
        let surface_tex = SURFACE.get().unwrap().get_current_texture()?;
        let view = surface_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_tex_read = &self.depth_texture.read().unwrap();
        let depth_view = &depth_tex_read.view;
        let bundles_read = self.static_cmds.read().unwrap();
        let bundles = bundles_read.iter().map(|(_idx, bundle)| bundle);

        let mut cmd_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Cmd Encoder"),
            });

        {
            let _rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Static Object Render Pass"),
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
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        let clearc_cmd = std::iter::once(cmd_encoder.finish());

        let mut cmd_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Cmd Encoder"),
            });

        {
            let mut rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Static Object Render Pass"),
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
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.execute_bundles(bundles);
        }

        let bundle_render_cmd = std::iter::once(cmd_encoder.finish());

        let mut contexts_write = self.render_ctxs.write().unwrap();

        let cmds_map = std::mem::take(&mut *contexts_write);

        let ids = cmds_map.into_iter().map(|(_, ctx)| ctx.id);

        todo!();

        let mut render_ctxs = std::mem::take(&mut *render_ctxs_write);

        let cmds_map: Vec<_> = ids.map(|id| render_ctxs.remove(&id).unwrap()).collect();

        let pipeline_db_read = self.pipeline_db.read().unwrap();

        let cmds: Vec<_> = cmds_map
            .into_par_iter()
            .map(|ctx| {
                let RenderCmd {
                    vertex_buffer,
                    index_buffer,
                    bind_groups,
                    pipeline,
                    draw_cmd,
                    ..
                } = ctx;

                let mut cmd_encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Cmd Encoder"),
                        });

                {
                    let mut rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Static Object Render Pass"),
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
                            view: depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    if let Some(pipeline) = pipeline {
                        let pipeline = match pipeline_db_read.pipelines.get(&pipeline) {
                            Some(pipeline) => pipeline,
                            None => pipeline_db_read.get_default(),
                        };

                        if let GpuPipeline::Render(pipeline) = pipeline {
                            rpass.set_pipeline(pipeline);
                        }
                    }
                    if let Some((slot, vertex_buf)) = vertex_buffer {
                        rpass.set_vertex_buffer(slot, vertex_buf.slice(..))
                    }

                    for (slot, bind_group, offsets) in bind_groups {
                        rpass.set_bind_group(slot, bind_group, offsets)
                    }

                    if let Some(index_buf) = index_buffer {
                        rpass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16)
                    }

                    if let Some(draw) = draw_cmd {
                        let DrawCmd {
                            indices,
                            base_vertex,
                            instances,
                        } = draw;
                        rpass.draw_indexed(indices, base_vertex, instances);
                    }
                }
                cmd_encoder.finish()
            })
            .collect();

        self.queue
            .submit(clearc_cmd.chain(bundle_render_cmd).chain(cmds));
        surface_tex.present();

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
