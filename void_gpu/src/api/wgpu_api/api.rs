use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;
use void_core::IBuilder;

use crate::{
    ContextDesc, IBindGroup, IBuffer, IContext, IRenderContext, TextureDesc, TextureError,
};

use crate::{
    CommandListIndex, Displayable, GpuResource, IGpu, IGpuCommandBuffer, IPipeline, Texture,
};

impl IBuffer for wgpu::CommandBuffer {}

impl IPipeline for wgpu::RenderPipeline {}
impl IPipeline for wgpu::ComputePipeline {}
impl IBuffer for wgpu::Buffer {}
impl IBindGroup for wgpu::BindGroup {}

pub struct RenderContext<'a> {
    encoder: wgpu::CommandEncoder,
    rpass: Option<wgpu::RenderPass<'a>>,
}

impl<'a> IContext<'a> for RenderContext<'a> {
    type Buffer = wgpu::CommandBuffer;
    fn new(gpu_resource: &GpuResource) -> Result<Self> {
        todo!()
    }
    fn end(mut self) -> Self::Buffer {
        if let Some(rpass) = self.rpass.take() {
            drop(rpass)
        }
        self.encoder.finish()
    }
}

impl<'a> IRenderContext<'a> for RenderContext<'a> {
    type BindGroup = wgpu::BindGroup;
    type Pipeline = wgpu::RenderPipeline;
    fn set_pipeline(&mut self, pipeline: &'a Self::Pipeline) {
        todo!()
    }
    fn draw(&mut self) {
        todo!()
    }
    fn set_bind_group(&mut self, index: u32, group: &'a Self::BindGroup) {
        todo!()
    }
    fn set_index_buffer(&mut self, buffer: &'a Self::Buffer) {
        todo!()
    }
    fn set_vertex_buffer(&mut self, slot: u32, buffer: &'a Self::Buffer) {
        todo!()
    }
    fn draw_instanced(&mut self, instances: std::ops::Range<u32>) {
        todo!()
    }
}

impl IGpuCommandBuffer for wgpu::CommandEncoder {}

pub struct Gpu<'a, T>
where
    T: Displayable<'a>,
{
    gpu_resource: Arc<GpuResource<'a, T>>,
    node_id: &'a [u8; 6],
    commands: BTreeMap<CommandListIndex, wgpu::CommandBuffer>,
}

impl<'a, T> IGpu<'a, T> for Gpu<'a, T>
where
    T: Displayable<'a>,
{
    type Texture = Texture;
    type RenderPipeline = wgpu::RenderPipeline;
    type RenderContext = RenderContext<'a>;
    type CmdBuffer = wgpu::CommandBuffer;
    type ComputePipeline = wgpu::ComputePipeline;
    type Err = GpuError;

    fn present(&self) {
        todo!()
    }
    fn create_texture(&self, texture_desc: TextureDesc) -> Result<Self::Texture, Self::Err> {
        let device = &self.gpu_resource.device;
        let queue = &self.gpu_resource.queue;
        let TextureDesc { data, .. } = texture_desc;
        let tex = Texture::from_bytes(device, queue, data, "Texture")?;
        Ok(tex)
    }

    fn begin_render_ctx(&mut self, idx: ContextDesc) -> Result<Self::RenderContext, Self::Err> {
        let encoder =
            self.gpu_resource
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder Descriptro"),
                });
        Ok(RenderContext {
            encoder,
            rpass: None,
        })
    }
    fn submit_ctx(&mut self, ctx: impl IContext<'a, Buffer = Self::CmdBuffer>) {
        let cmd_buffer = ctx.end();
        let idx = CommandListIndex::new(&self.node_id);
        self.commands.insert(idx, cmd_buffer);
    }
    fn create_pipeline(
        &self,
        shader_src: &str,
        pipeline_builder: impl IBuilder<Output = Self::RenderPipeline>,
    ) -> Result<Self::RenderPipeline, Self::Err> {
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum GpuError {
    #[error("Error creating texture {0}")]
    TextureError(#[from] TextureError),
    #[error("Non existat command encoder at")]
    NonExistantEncoder,
}
