use wgpu::{util::RenderEncoder, Operations};

use crate::{create_render_pipeline, gpu::Gpu, texture};

pub struct HdrPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture: texture::Texture,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
}

impl HdrPipeline {
    pub fn new(gpu: &Gpu) -> Self {
        let config = gpu.get_config();
        let width = config.width;
        let height = config.height;

        let device = &gpu.device;

        let format = wgpu::TextureFormat::Rgba16Float;

        let texture = texture::Texture::create_2d_texture(
            gpu,
            width,
            height,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("Hdr::texture"),
        );

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hdr::layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hdr::bind_group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        let shader = wgpu::include_wgsl!("hdr.wgsl");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = create_render_pipeline(
            gpu,
            &pipeline_layout,
            config.format.add_srgb_suffix(),
            None,
            &[],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
        );

        Self {
            pipeline,
            bind_group,
            texture,
            width,
            height,
            format,
            layout,
        }
    }

    pub fn resize(&mut self, gpu: &Gpu, width: u32, height: u32) {
        let device = &gpu.device;

        self.texture = texture::Texture::create_2d_texture(
            gpu,
            height,
            width,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("Hdr::texture"),
        );

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hdr::bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.texture.sampler),
                },
            ],
        });

        self.width = width;
        self.height = height;
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Hdr::process"),
            occlusion_query_set: None,
            timestamp_writes: None,
            depth_stencil_attachment: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
