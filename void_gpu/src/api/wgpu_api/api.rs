use super::texture::{Texture, TextureError};
use crate::api::{
    CommandListIndex, Displayable, DrawModel, GpuPipeline, IBindGroup, IBuffer, IContext, IGpu,
    IGpuCommandBuffer, IPipeline, PipelineId,
};
use rand::Rng;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Range,
    sync::{Arc, RwLock},
};
use thiserror::Error;
use uuid::Uuid;
use void_core::rayon::iter::{IntoParallelRefIterator, ParallelIterator};

impl IBuffer for wgpu::CommandBuffer {}
impl IPipeline for wgpu::RenderPipeline {}
impl IPipeline for wgpu::ComputePipeline {}
impl IBuffer for wgpu::Buffer {}
impl IBindGroup for wgpu::BindGroup {}
impl IBuffer for wgpu::RenderBundle {}
impl IGpuCommandBuffer for wgpu::CommandEncoder {}

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
    depth_texture: Option<Texture>,
    gpu: Arc<Gpu<'a, T>>,
    pipeline_id: Option<PipelineId>,
}

impl<'a, T: Displayable<'a>> StaticRenderCtx<'a, T> {
    pub fn new(gpu: Arc<Gpu<'a, T>>) -> Self {
        Self {
            render_bundles: RwLock::new(Vec::new()),
            depth_texture: None,
            gpu,
            pipeline_id: None,
        }
    }
}

impl<'a, 'b, T: Displayable<'a>> DrawModel<'b, 'a> for StaticRenderCtx<'a, T> {
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
    depth_tex: Option<Texture>,
}

pub enum CtxOutput {
    Upload,
    Render(RenderCmd),
}

impl<'a, T: Displayable<'a>> IContext for StaticRenderCtx<'a, T> {
    type Output = CtxOutput;
    type Encoder = CtxInterm<'a>;

    fn set_pileine(&mut self, id: PipelineId) {
        self.pipeline_id = Some(id);
    }

    fn set_stage(&self, enc: Self::Encoder) {
        let mut bundle_encoder =
            self.gpu
                .device
                .create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: Some("Render Bundle Encoder"),
                    color_formats: &[Some(self.gpu.config.read().unwrap().format)],
                    depth_stencil: None,
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
            depth_tex: self.depth_texture,
            pipeline_id,
        })
    }
}

pub struct StaticRenderObject {
    depth_tex: Option<Texture>,
    bundle: Vec<wgpu::RenderBundle>,
    pipeline_id: Option<PipelineId>,
}

pub struct DynamicRenderObject {
    depth_tex: Option<Texture>,
    cmd: wgpu::CommandBuffer,
    pipeline_id: Option<PipelineId>,
}

pub struct Gpu<'a, T>
where
    T: Displayable<'a>,
{
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) device: wgpu::Device,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: RwLock<wgpu::SurfaceConfiguration>,
    pub window: Arc<T>,
    node_id: [u8; 6],
    static_cmds: RwLock<BTreeMap<CommandListIndex, StaticRenderObject>>,
    dynamic_cmds: RwLock<BTreeMap<CommandListIndex, DynamicRenderObject>>,
    render_pileines: RwLock<HashMap<PipelineId, wgpu::RenderPipeline>>,
    compute_pipelines: RwLock<HashMap<PipelineId, wgpu::ComputePipeline>>,
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

        Arc::new(Self {
            adapter,
            surface,
            device,
            queue,
            config: RwLock::new(config),
            window,
            node_id: rand::thread_rng().gen::<[u8; 6]>(),
            static_cmds: RwLock::new(BTreeMap::new()),
            dynamic_cmds: RwLock::new(BTreeMap::new()),
            render_pileines: RwLock::new(HashMap::new()),
            compute_pipelines: RwLock::new(HashMap::new()),
        })
    }
}

impl<'a, T> IGpu for Gpu<'a, T>
where
    T: Displayable<'a> + 'a,
{
    type CtxOutput = CtxOutput;
    type Err = GpuError;

    fn window_update(&self, width: u32, height: u32) {
        let mut config_write = self.config.write().unwrap();
        config_write.width = width;
        config_write.height = height;
        self.surface.configure(&self.device, &config_write);
    }

    fn insert_pipeline(&self, pipeline: crate::api::GpuPipeline) -> PipelineId {
        let id = PipelineId(Uuid::new_v4());
        match pipeline {
            GpuPipeline::Render(pipeline) => {
                self.render_pileines.write().unwrap().insert(id, pipeline);
            }
            GpuPipeline::Compute(pipeline) => {
                self.compute_pipelines.write().unwrap().insert(id, pipeline);
            }
        }
        id
    }

    fn submit_ctx_output(&self, ctxs: impl Iterator<Item = Self::CtxOutput>) {
        let _ = ctxs.map(|out| match out {
            CtxOutput::Render(render_cmd) => {
                let depth_tex = render_cmd.depth_tex;
                let render_type = render_cmd.render_type;
                let pipeline_id = render_cmd.pipeline_id;
                match render_type {
                    RenderType::Single(cmd) => {
                        self.dynamic_cmds.write().unwrap().insert(
                            CommandListIndex::new(&self.node_id),
                            DynamicRenderObject {
                                depth_tex,
                                cmd,
                                pipeline_id,
                            },
                        );
                    }
                    RenderType::Bundled(bundle) => {
                        self.static_cmds.write().unwrap().insert(
                            CommandListIndex::new(&self.node_id),
                            StaticRenderObject {
                                depth_tex,
                                bundle,
                                pipeline_id,
                            },
                        );
                    }
                }
            }
            CtxOutput::Upload => {}
        });
    }

    fn present(&self) -> Result<(), Self::Err> {
        let surface_tex = self.surface.get_current_texture()?;
        let view = surface_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("Surface Texture View"),
                format: Some(self.config.read().unwrap().format),
                ..Default::default()
            });

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
                depth_stencil_attachment: None,
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
                let pipeline_id = obj.pipeline_id.unwrap();
                let pipelines_read = self.render_pileines.read().unwrap();
                let pipeline = pipelines_read.get(&pipeline_id).unwrap();
                let mut cmd_encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Cmd Encoder"),
                        });

                {
                    let mut rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Static Object Render Pass"),
                        color_attachments: &[],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    rpass.set_pipeline(pipeline);
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

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("{0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
}
