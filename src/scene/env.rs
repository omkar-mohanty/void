use crate::{
    gpu::Gpu,
    resource::{self, HdrLoader},
    texture::CubeTexture,
};

static ENV_BIND_GROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("hdr::environmetnt_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    };

pub struct Environment {
    skybox: CubeTexture,
}

impl Environment {
    pub async fn new(loader: HdrLoader, gpu: &Gpu) -> anyhow::Result<Self> {
        let data = crate::io::fs::load_inbuilt_binary("pure-sky.hdr").await?;

        let skybox = loader.from_equirectangular_bytes(
            gpu,
            &data,
            1080,
            Some("Default pure sky environment"),
        )?;

        Ok(Self { skybox })
    }

    pub fn load(&self, gpu: &Gpu) -> anyhow::Result<()> {
        let device = &gpu.device;
        let bind_group_layout = device.create_bind_group_layout(&ENV_BIND_GROUP_LAYOUT_DESC);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("environment_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.skybox.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.skybox.sampler()),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("env encoder"),
        });

        {
        
        }

        Ok(())
    }
}
