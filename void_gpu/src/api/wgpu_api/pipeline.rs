use std::sync::Arc;

use void_core::IBuilder;

use crate::{
    api::{Gpu, Texture},
    model::{ModelVertex, Vertex},
};

use super::Displayable;

pub struct PipelineBuilder<'a, T: Displayable<'a>> {
    gpu: Arc<Gpu<'a, T>>,
    shader_src: Option<&'a str>,
    depth_format: Option<Texture>,
}

impl<'a, T: Displayable<'a>> PipelineBuilder<'a, T> {
    pub fn new(gpu: Arc<Gpu<'a, T>>) -> Self {
        Self {
            gpu,
            shader_src: None,
            depth_format: None,
        }
    }
    pub fn set_shader(&mut self, shader_src: &'a str) {
        self.shader_src = Some(shader_src)
    }
    pub fn set_depth_format(&mut self, depth_format: Texture) {
        self.depth_format = Some(depth_format);
    }
}

impl<'a, T: Displayable<'a>> IBuilder for PipelineBuilder<'a, T> {
    type Output = wgpu::RenderPipeline;

    async fn build(self) -> void_core::Result<Self::Output, void_core::BuilderError> {
        let shader_src = self.shader_src.unwrap_or(include_str!("shader.wgsl"));
        let device = &self.gpu.device;
        let color_format = self.gpu.config.read().unwrap().format;
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Model Renderer Pipeline"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        };
        Ok(create_render_pipeline(
            &self.gpu,
            &layout,
            color_format,
            None,
            &[ModelVertex::desc()],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
        ))
    }
}

pub fn create_render_pipeline<'a, T: Displayable<'a>>(
    device: &Gpu<'a, T>,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    topology: wgpu::PrimitiveTopology, // NEW!
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let device = &device.device;
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{:?}", shader)),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
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
