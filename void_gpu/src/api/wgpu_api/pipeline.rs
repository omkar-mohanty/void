use std::sync::{Arc, OnceLock};

use void_core::IBuilder;

use crate::{
    api::{Gpu, GpuPipeline, IPipeline, PipelineType, Texture},
    model::{ModelVertex, Vertex},
};

impl IPipeline for wgpu::RenderPipeline {}

pub(crate) static CAMERA_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();
pub(crate) static TEXTURE_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

/// Compute shader entry point name
static COMPUTE_ENTRY: &'static str = "cs_main";
/// Vertex shader entry point name
static VERTEX_ENTRY: &'static str = "vs_main";
/// Fragment shader entry point name
static FRAGMENT_ENTRY: &'static str = "fs_main";

/// Default shader source
static DEFAULT_RENDER_SHADER: &'static str = include_str!("shader.wgsl");

/// Texture [```wgpu::BindGroupLayoutDescriptor]
pub(crate) static TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor =
    wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    };

/// Camera [```wgpu::BindGroupLayoutDescriptor```]
pub(crate) static CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor =
    wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("camera_bind_group_layout"),
    };

pub struct PipelineBuilder<'a> {
    gpu: Arc<Gpu>,
    shader_src: Option<String>,
    bind_group_layout_descriptros: Option<Vec<wgpu::BindGroupLayoutDescriptor<'a>>>,
    pipeline_type: PipelineType,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(gpu: Arc<Gpu>, pipeline_type: PipelineType) -> Self {
        Self {
            gpu,
            shader_src: None,
            pipeline_type,
            bind_group_layout_descriptros: None,
        }
    }
    pub fn set_shader(&mut self, shader_src: &'a str) {
        self.shader_src = Some(shader_src)
    }
    pub fn add_layout_descriptors(
        &mut self,
        layouts: impl Iterator<Item = wgpu::BindGroupLayoutDescriptor<'a>>,
    ) {
        let descs = self.bind_group_layout_descriptros.take();

        if let Some(mut desc) = descs {
            desc.extend(layouts);
            self.bind_group_layout_descriptros = Some(desc);
        } else {
            self.bind_group_layout_descriptros = Some(Vec::new());
        }
    }
}

impl<'a> IBuilder for PipelineBuilder<'a> {
    type Output = GpuPipeline;

    async fn build(self) -> void_core::Result<Self::Output, void_core::BuilderError> {
        let shader_src = self.shader_src.unwrap_or(DEFAULT_RENDER_SHADER);
        let device = &self.gpu.device;
        let color_format = self.gpu.config.read().unwrap().format;

        let mut layout_descriptors = vec![
            &TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR,
            &CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR,
        ];

        let descs = self.bind_group_layout_descriptros.unwrap_or(vec![]);

        let descs = descs.iter();

        layout_descriptors.extend(descs);

        let bind_group_layouts: Vec<_> = layout_descriptors
            .into_iter()
            .map(|descriptor| device.create_bind_group_layout(descriptor))
            .collect();

        let refs: Vec<_> = bind_group_layouts.iter().collect();

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Model Renderer Pipeline"),
            bind_group_layouts: &refs,
            push_constant_ranges: &[],
        });
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        };

        let pipeline = match self.pipeline_type {
            PipelineType::Render => {
                let pipeline = create_render_pipeline(
                    &self.gpu.device,
                    &layout,
                    color_format,
                    Some(Texture::DEPTH_FORMAT),
                    &[ModelVertex::desc()],
                    wgpu::PrimitiveTopology::TriangleList,
                    shader,
                );

                GpuPipeline::Render(pipeline)
            }
            PipelineType::Compute => {
                let pipeline = create_compute_pipeline(&self.gpu, &layout, shader);
                GpuPipeline::Compute(pipeline)
            }
        };

        Ok(pipeline)
    }
}

pub(crate) fn default_render_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> GpuPipeline {
    let shader_src = DEFAULT_RENDER_SHADER;
    let color_format = config.format;

    let layout_descriptors = vec![
        &TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR,
        &CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR,
    ];

    let mut bind_group_layouts: Vec<_> = layout_descriptors
        .into_iter()
        .map(|descriptor| device.create_bind_group_layout(descriptor))
        .collect();

    let refs: Vec<_> = bind_group_layouts.iter().collect();

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Model Renderer Pipeline"),
        bind_group_layouts: &refs,
        push_constant_ranges: &[],
    });
    let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    };

    let pipeline = create_render_pipeline(
        &device,
        &layout,
        color_format,
        Some(Texture::DEPTH_FORMAT),
        &[ModelVertex::desc()],
        wgpu::PrimitiveTopology::TriangleList,
        shader,
    );

    TEXTURE_BIND_GROUP_LAYOUT.get_or_init(|| bind_group_layouts.remove(0));
    CAMERA_BIND_GROUP_LAYOUT.get_or_init(|| bind_group_layouts.remove(1));

    GpuPipeline::Render(pipeline)
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    topology: wgpu::PrimitiveTopology, // NEW!
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{:?}", shader)),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: VERTEX_ENTRY,
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: FRAGMENT_ENTRY,
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology, // NEW!
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual, // UDPATED!
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    })
}

fn create_compute_pipeline(
    device: &Gpu,
    layout: &wgpu::PipelineLayout,
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::ComputePipeline {
    let device = &device.device;
    let shader = device.create_shader_module(shader);

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Shader"),
        layout: Some(layout),
        module: &shader,
        entry_point: COMPUTE_ENTRY,
    });

    pipeline
}
