use super::pipeline::default_render_pipeline;
use super::texture::{Texture, TextureError};
use crate::api::{CommandListIndex, Displayable, DrawModel, IContext, IGpu, PipelineId};
use rand::Rng;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Range,
    sync::{Arc, RwLock},
};
use thiserror::Error;
use uuid::{uuid, Uuid};
use void_core::rayon::iter::{IntoParallelRefIterator, ParallelIterator};

static DEFAULT_PIPELINE_ID: PipelineId = PipelineId(uuid!("5b929ea6-7547-4e70-89a0-6f9ad7f9ffe4"));

trait IEncoder<'a> {
    fn set_render_pipeline(&mut self, pipeline: &'a wgpu::RenderPipeline);
    fn set_compute_pipeline(&mut self, pipeline: &'a wgpu::ComputePipeline);
}

impl<'a> IEncoder<'a> for wgpu::RenderBundleEncoder<'a> {
    fn set_render_pipeline(&mut self, pipeline: &'a wgpu::RenderPipeline) {
        self.set_pipeline(pipeline);
    }

    fn set_compute_pipeline(&mut self, _pipeline: &wgpu::ComputePipeline) {}
}

impl<'a> IEncoder<'a> for wgpu::RenderPass<'a> {
    fn set_render_pipeline(&mut self, pipeline: &'a wgpu::RenderPipeline) {
        self.set_pipeline(pipeline);
    }

    fn set_compute_pipeline(&mut self, _pipeline: &wgpu::ComputePipeline) {}
}

impl<'a> IEncoder<'a> for wgpu::ComputePass<'a> {
    fn set_render_pipeline(&mut self, _pipeline: &'a wgpu::RenderPipeline) {}

    fn set_compute_pipeline(&mut self, pipeline: &'a wgpu::ComputePipeline) {
        self.set_pipeline(pipeline);
    }
}

pub struct Buffer<'a> {
    slot: u32,
    buffer: &'a wgpu::Buffer,
}

pub struct BindGroup<'a> {
    index: u32,
    offset: &'a [wgpu::DynamicOffset],
    bind_group: &'a wgpu::BindGroup,
}

pub enum CtxInterm<'a> {
    Render {
        vertex_buffer: Option<Buffer<'a>>,
        bind_group: Option<BindGroup<'a>>,
        camera_bind_group: Option<BindGroup<'a>>,
        index_buffer: Option<Buffer<'a>>,
        instances: Range<u32>,
        num_elements: Range<u32>,
    },
    Upload {},
}

pub struct StaticRenderCtx<'a, T: Displayable<'a>> {
    render_bundles: RwLock<Vec<wgpu::RenderBundle>>,
    gpu: Arc<Gpu<'a, T>>,
    pipeline_id: Option<PipelineId>,
}

impl<'a, T: Displayable<'a>> StaticRenderCtx<'a, T> {
    pub fn new(gpu: Arc<Gpu<'a, T>>) -> Self {
        Self {
            render_bundles: RwLock::new(Vec::new()),
            gpu,
            pipeline_id: None,
        }
    }
}

impl<'a, 'b, T: Displayable<'a> + 'a> DrawModel<'b, 'a> for StaticRenderCtx<'a, T> {
    type BindGroup = wgpu::BindGroup;

    fn draw_mesh(
        &self,
        mesh: &'a super::model::Mesh,
        material: &'a super::model::Material,
        camera_bind_group: &'a Self::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group)
    }
    fn draw_model(&self, model: &'a super::model::Model, camera_bind_group: &'a Self::BindGroup) {
        self.draw_model_instanced(model, 0..1, camera_bind_group)
    }
    fn draw_mesh_instanced(
        &self,
        mesh: &'a super::model::Mesh,
        material: &'a super::model::Material,
        instances: Range<u32>,
        camera_bind_group: &'a Self::BindGroup,
    ) {
        let vertex_buffer = Some(Buffer {
            slot: 0,
            buffer: &mesh.vertex_buffer,
        });
        let bind_group = Some(BindGroup {
            index: 0,
            bind_group: &material.bind_group,
            offset: &[],
        });
        let camera_bind_group = Some(BindGroup {
            index: 1,
            bind_group: camera_bind_group,
            offset: &[],
        });
        let index_buffer = Some(Buffer {
            slot: 0,
            buffer: &mesh.index_buffer,
        });
        let interm = CtxInterm::Render {
            vertex_buffer,
            bind_group,
            camera_bind_group,
            index_buffer,
            instances,
            num_elements: 0..mesh.num_elements,
        };
        self.set_stage(interm);
    }
    fn draw_model_instanced(
        &self,
        model: &'a super::model::Model,
        instances: Range<u32>,
        camera_bind_group: &'a Self::BindGroup,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group);
        }
    }
    fn draw_mesh_nbd(&self, mesh: &'a crate::model::Mesh, material: &'a crate::model::Material) {
        self.draw_mesh_nbd_instanced(mesh, material, 0..1);
    }
    fn draw_mesh_nbd_instanced(
        &self,
        mesh: &'a crate::model::Mesh,
        material: &'a crate::model::Material,
        instances: Range<u32>,
    ) {
        let vertex_buffer = Some(Buffer {
            slot: 0,
            buffer: &mesh.vertex_buffer,
        });
        let bind_group = Some(BindGroup {
            index: 0,
            bind_group: &material.bind_group,
            offset: &[],
        });
        let camera_bind_group = None;
        let index_buffer = Some(Buffer {
            slot: 0,
            buffer: &mesh.index_buffer,
        });
        let interm = CtxInterm::Render {
            vertex_buffer,
            bind_group,
            camera_bind_group,
            index_buffer,
            instances,
            num_elements: 0..mesh.num_elements,
        };
        self.set_stage(interm);
    }
    fn draw_model_nbd(&self, model: &'a crate::model::Model) {
        self.draw_model_nbd_instanced(model, 0..1);
    }
    fn draw_model_nbd_instanced(&self, model: &'a crate::model::Model, instances: Range<u32>) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_nbd_instanced(mesh, material, instances.clone());
        }
    }
}

pub enum RenderType {
    Bundled(Vec<wgpu::RenderBundle>),
    Single(wgpu::CommandBuffer),
}

pub struct RenderCmd {
    render_type: RenderType,
    pipeline_id: Option<PipelineId>,
}

pub enum CtxOutput {
    Upload,
    Render(RenderCmd),
}

impl<'a, T: Displayable<'a> + 'a> IContext for StaticRenderCtx<'a, T> {
    type Output = CtxOutput;
    type Encoder = CtxInterm<'a>;

    fn set_pileine(&mut self, id: PipelineId) {
        self.pipeline_id = Some(id);
    }

    fn set_stage(&self, enc: Self::Encoder) {
        let pipeline_read = self.gpu.pipeline_db.read().unwrap();

        let mut bundle_encoder =
            self.gpu
                .device
                .create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: Some("Render Bundle Encoder"),
                    color_formats: &[Some(self.gpu.config.read().unwrap().format)],
                    depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                        format: Texture::DEPTH_FORMAT,
                        depth_read_only: false,
                        stencil_read_only: true,
                    }),
                    sample_count: 1,
                    multiview: None,
                });

        match enc {
            CtxInterm::Render {
                vertex_buffer,
                bind_group,
                index_buffer,
                instances,
                num_elements,
                camera_bind_group,
            } => {
                if let Some(vertex_buf) = vertex_buffer {
                    bundle_encoder.set_vertex_buffer(vertex_buf.slot, vertex_buf.buffer.slice(..));
                }
                if let Some(bind_group) = bind_group {
                    bundle_encoder.set_bind_group(
                        bind_group.index,
                        bind_group.bind_group,
                        bind_group.offset,
                    );
                }
                if let Some(index_buf) = index_buffer {
                    bundle_encoder
                        .set_index_buffer(index_buf.buffer.slice(..), wgpu::IndexFormat::Uint16);
                }
                if let Some(camera_bind_group) = camera_bind_group {
                    bundle_encoder.set_bind_group(
                        camera_bind_group.index,
                        camera_bind_group.bind_group,
                        camera_bind_group.offset,
                    );
                }
                pipeline_read.set_pipeline(
                    self.pipeline_id.unwrap_or(DEFAULT_PIPELINE_ID),
                    &mut bundle_encoder,
                );
                bundle_encoder.draw_indexed(num_elements, 0, instances);
            }
            _ => {}
        }

        let bundle = bundle_encoder.finish(&wgpu::RenderBundleDescriptor { label: None });
        self.render_bundles.write().unwrap().push(bundle);
    }

    fn end(mut self) -> Self::Output {
        let bundles = std::mem::take(self.render_bundles.get_mut().unwrap());
        let pipeline_id = self.pipeline_id;
        CtxOutput::Render(RenderCmd {
            render_type: RenderType::Bundled(bundles),
            pipeline_id,
        })
    }
}

pub struct StaticRenderObject {
    bundle: Vec<wgpu::RenderBundle>,
    pipeline_id: Option<PipelineId>,
}

#[allow(dead_code)]
pub struct DynamicRenderObject {
    cmd: wgpu::CommandBuffer,
    pipeline_id: Option<PipelineId>,
}

pub enum GpuPipeline {
    Render(wgpu::RenderPipeline),
    Compute(wgpu::ComputePipeline),
}

pub struct PipelineDB {
    pipelines: HashMap<PipelineId, GpuPipeline>,
}

impl PipelineDB {
    pub async fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let pipeline = default_render_pipeline(device, config);
        let mut pipelines = HashMap::new();

        pipelines.insert(DEFAULT_PIPELINE_ID, pipeline);
        Self { pipelines }
    }

    pub fn insert_pipeline(&mut self, pipeline: GpuPipeline) -> PipelineId {
        let id = PipelineId(Uuid::new_v4());
        self.pipelines.insert(id, pipeline);
        id
    }

    pub fn set_pipeline<'a>(&'a self, id: PipelineId, encoder: &mut dyn IEncoder<'a>) {
        let pipeline = self.pipelines.get(&id).unwrap();
        match pipeline {
            GpuPipeline::Render(pipeline) => encoder.set_render_pipeline(pipeline),
            GpuPipeline::Compute(_) => {}
        };
    }
}

pub struct Gpu<'a, T>
where
    T: Displayable<'a>,
{
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: RwLock<wgpu::SurfaceConfiguration>,
    pub window: Arc<T>,
    node_id: [u8; 6],
    static_cmds: RwLock<BTreeMap<CommandListIndex, StaticRenderObject>>,
    dynamic_cmds: RwLock<BTreeMap<CommandListIndex, DynamicRenderObject>>,
    pipeline_db: RwLock<PipelineDB>,
    depth_texture: RwLock<Texture>,
}

impl<'a, T: Displayable<'a> + 'a> Gpu<'a, T> {
    pub async fn new(window: Arc<T>, width: u32, height: u32) -> Arc<Self> {
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

        let depth_texture = RwLock::new(Texture::create_depth_texture(
            &device,
            &config,
            "Depth Texture",
        ));

        let pipeline_db = RwLock::new(PipelineDB::new(&device, &config).await);

        Arc::new(Self {
            surface,
            device,
            queue,
            config: RwLock::new(config),
            window,
            node_id: rand::thread_rng().gen::<[u8; 6]>(),
            static_cmds: RwLock::new(BTreeMap::new()),
            dynamic_cmds: RwLock::new(BTreeMap::new()),
            pipeline_db,
            depth_texture,
        })
    }
}

impl<'a, T> IGpu for Gpu<'a, T>
where
    T: Displayable<'a> + 'a,
{
    type CtxOutput = CtxOutput;
    type Err = GpuError;
    type Pipeline = GpuPipeline;

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
        self.surface.configure(&self.device, &config_write);
    }

    fn insert_pipeline(&self, pipeline: GpuPipeline) -> PipelineId {
        self.pipeline_db.write().unwrap().insert_pipeline(pipeline)
    }

    fn submit_ctx_output(&self, ctxs: impl Iterator<Item = Self::CtxOutput>) {
        for ctx in ctxs {
            match ctx {
                CtxOutput::Render(render_cmd) => {
                    let render_type = render_cmd.render_type;
                    let pipeline_id = render_cmd.pipeline_id;
                    match render_type {
                        RenderType::Single(cmd) => {
                            self.dynamic_cmds.write().unwrap().insert(
                                CommandListIndex::new(&self.node_id),
                                DynamicRenderObject { cmd, pipeline_id },
                            );
                        }
                        RenderType::Bundled(bundle) => {
                            self.static_cmds.write().unwrap().insert(
                                CommandListIndex::new(&self.node_id),
                                StaticRenderObject {
                                    bundle,
                                    pipeline_id,
                                },
                            );
                        }
                    }
                }
                CtxOutput::Upload => {}
            }
        }
    }

    fn present(&self) -> Result<(), Self::Err> {
        let surface_tex = self.surface.get_current_texture()?;
        let view = surface_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut cmd_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Cmd Encoder"),
            });

        let depth_tex_read = &self.depth_texture.read().unwrap();
        let depth_view = &depth_tex_read.view;

        let pipeline_db = self.pipeline_db.read().unwrap();

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
        }

        let clear_cmd = std::iter::once(cmd_encoder.finish());

        let cmds: Vec<_> = self
            .static_cmds
            .read()
            .unwrap()
            .par_iter()
            .map(|(_, obj)| {
                let bundles = &obj.bundle;
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
                    rpass.execute_bundles(bundles.iter());
                }
                cmd_encoder.finish()
            })
            .collect();

        self.queue.submit(clear_cmd.chain(cmds));
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
