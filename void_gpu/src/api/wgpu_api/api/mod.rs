use render_ctx::{DrawCmd, RenderCtx};
use wgpu::util::DeviceExt;

use self::upload_ctx::UploadCtx;

use super::texture::{Texture, TextureError};
use crate::api::{BufferId, CommandListIndex, Displayable, IBuffer, ICtxOut, IGpu, PipelineId};
use rand::Rng;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};
use thiserror::Error;
use uuid::{uuid, Uuid};
use void_core::rayon::iter::{IntoParallelIterator, ParallelIterator};

pub mod render_ctx;
pub mod upload_ctx;

impl IBuffer for wgpu::Buffer {}

static DEFAULT_RENDER_PIPELINE: PipelineId =
    PipelineId(uuid!("5b929ea6-7547-4e70-89a0-6f9ad7f9ffe4"));

pub(crate) static DEFAULT_CAMERA_BUFFER_ID: BufferId =
    BufferId(uuid!("5b929ea6-7547-4e70-89a0-6f9ad7f9ffe4"));

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

pub enum CtxOut<'a, 'b> {
    Render(RenderCtx<'a, 'b>),
    Update(UploadCtx<'a, 'b>),
}

impl ICtxOut for CtxOut<'_, '_> {}

pub enum GpuPipeline {
    Render(wgpu::RenderPipeline),
    Compute(wgpu::ComputePipeline),
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
    render_ctxs: RwLock<BTreeMap<CommandListIndex, RenderCtx<'a, 'a>>>,
    static_cmds: RwLock<BTreeMap<CommandListIndex, wgpu::RenderBundle>>,
    pub(crate) buffers: RwLock<HashMap<BufferId, wgpu::Buffer>>,
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

        let mut buffers = HashMap::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::UNIFORM,
        });

        buffers.insert(DEFAULT_CAMERA_BUFFER_ID, camera_buffer);

        Arc::new(Self {
            surface,
            device,
            queue,
            config: RwLock::new(config),
            window,
            node_id: rand::thread_rng().gen::<[u8; 6]>(),
            render_ctxs: RwLock::new(BTreeMap::new()),
            static_cmds: RwLock::new(BTreeMap::new()),
            buffers: RwLock::new(buffers),
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

impl<'a, 'b, T> IGpu for Gpu<'a, T>
where
    T: Displayable<'a> + 'a,
{
    type Err = GpuError;
    type CtxOut = CtxOut<'a, 'a>;
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
        self.surface.configure(&self.device, &config_write);
    }

    fn submit_ctx_out(&self, out: Self::CtxOut) {
        let mut render_ctxs_write = self.render_ctxs.write().unwrap();
        match out {
            CtxOut::Render(ctx) => {
                render_ctxs_write.insert(CommandListIndex::new(&self.node_id), ctx);
            }
            CtxOut::Update(ctx) => self.update_buffers(ctx),
        };
    }

    fn present(&self) -> Result<(), Self::Err> {
        let surface_tex = self.surface.get_current_texture()?;
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

        let mut cmds_write = self.render_ctxs.write().unwrap();

        let cmds_map = std::mem::take(&mut *cmds_write);

        let cmds: Vec<_> = cmds_map
            .into_par_iter()
            .map(|(_, ctx)| {
                let RenderCtx {
                    bind_groups,
                    vertex_buffer,
                    index_buffer,
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
                        rpass.set_pipeline(pipeline)
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
