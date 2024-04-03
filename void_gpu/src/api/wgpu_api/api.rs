use super::texture::{Texture, TextureError};
use crate::api::{
    CommandListIndex, Displayable, IBindGroup, IBuffer, IContext, IEncoder, IGpu,
    IGpuCommandBuffer, IPipeline, IRenderContext, IRenderEncoder,
};
use rand::Rng;
use std::{collections::BTreeMap, ops::Range, sync::Arc};
use thiserror::Error;

type CtxClosure<'a, T> = Box<dyn FnMut(Arc<T>) -> wgpu::CommandBuffer + 'a>;

impl IBuffer for wgpu::CommandBuffer {}
impl IPipeline for wgpu::RenderPipeline {}
impl IPipeline for wgpu::ComputePipeline {}
impl IBuffer for wgpu::Buffer {}
impl IBindGroup for wgpu::BindGroup {}
impl IBuffer for wgpu::RenderBundle {}
impl IGpuCommandBuffer for wgpu::CommandEncoder {}

impl<'a> IEncoder for wgpu::RenderBundleEncoder<'a> {
    type Buffer = wgpu::Buffer;
    type Pipeline = wgpu::RenderPipeline;
    type BindGroup = wgpu::BindGroup;
}

impl<'a> IRenderEncoder<'a> for wgpu::RenderBundleEncoder<'a> {
    fn set_pipeline(&mut self, pipeline: &'a Self::Pipeline) {
        self.set_pipeline(&pipeline);
    }
    fn set_bind_group(&mut self, index: u32, group: &'a Self::BindGroup) {
        self.set_bind_group(index, group, &[]);
    }
    fn draw(&mut self, verts: Range<u32>, instances: Range<u32>) {
        self.draw(verts, instances);
    }
    fn set_index_buffer(&mut self, buffer: &'a Self::Buffer) {
        self.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
    }
    fn set_vertex_buffer(&mut self, slot: u32, buffer: &'a Self::Buffer) {
        self.set_vertex_buffer(slot, buffer.slice(..))
    }
}

pub struct RenderContext<'a, T: Displayable<'a>> {
    gpu: Arc<Gpu<'a, T>>,
    render_bundles: Vec<wgpu::RenderBundle>,
    depth_texture: Option<Texture>,
}

impl<'a, T: Displayable<'a>> RenderContext<'a, T> {
    pub(crate) fn new(gpu: Arc<Gpu<'a, T>>) -> Self {
        Self {
            gpu,
            render_bundles: Vec::new(),
            depth_texture: None,
        }
    }
}

enum CtxOutput<'a, T> {
    Render(CtxClosure<'a, T>),
    Compute(CtxClosure<'a, T>),
}

impl<'a, T: Displayable<'a> + 'a> IContext<'a> for RenderContext<'a, T> {
    type Output = CtxOutput<'a, wgpu::TextureView>;
    type Encoder = wgpu::RenderBundleEncoder<'a>;

    fn get_encoder<'b>(&'a self) -> Self::Encoder
    where
        'a: 'b,
    {
        self.gpu
            .device
            .create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                label: Some("Render Bundle Encoder"),
                color_formats: &[],
                depth_stencil: None,
                sample_count: 1,
                multiview: None,
            })
    }

    fn set_encoder(&mut self, encoder: Self::Encoder) {
        let buncles = encoder.finish(&wgpu::RenderBundleDescriptor {
            label: Some("Render Bundle Encoder"),
        });
        self.render_bundles.push(buncles);
    }

    fn end(self) -> Self::Output {
        let bundles = self.render_bundles;

        let gpu = Arc::clone(&self.gpu);
        let boxxed = Box::new(move |tex_view| {
            let mut cmd_encoder =
                gpu.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Command Encoder"),
                    });
            {
                let mut rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Context Render Pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                rpass.execute_bundles(bundles.iter())
            }
            cmd_encoder.finish()
        });

        CtxOutput::Render(boxxed)
    }
}

impl<'a, T: Displayable<'a> + 'static> IRenderContext<'a, wgpu::RenderBundleEncoder<'a>>
    for RenderContext<'a, T>
{
}

pub struct Gpu<'a, T>
where
    T: Displayable<'a>,
{
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) device: wgpu::Device,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) window: Arc<T>,
    node_id: [u8; 6],
    render_bundles: Vec<wgpu::RenderBundle>,
    commands: BTreeMap<CommandListIndex, wgpu::CommandBuffer>,
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
            config,
            window,
            commands: BTreeMap::new(),
            node_id: rand::thread_rng().gen::<[u8; 6]>(),
            render_bundles: Vec::new(),
        })
    }
}

impl<'a, T> IGpu for Gpu<'a, T>
where
    T: Displayable<'a> + 'a,
{
    type CtxOutput = CtxOutput<'a, wgpu::TextureView>;
    type Err = GpuError;

    fn submit_ctx_output(
        &mut self,
        ctxs: impl Iterator<Item = Self::CtxOutput>,
    ) -> Result<(), Self::Err> {
        let tex = self.surface.get_current_texture()?;
        let tex_view = Arc::new(tex.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        }));
    
        let enco = ctxs.map(|out| {
            match out {
                CtxOutput::Render(mut clos) | CtxOutput::Compute(mut clos) => clos(Arc::clone(&tex_view)),
            }
        });
        self.queue.submit(enco);
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
