use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;
use void_core::IBuilder;
use wgpu::BufferSlice;

use crate::{IBindGroup, IBuffer, IContext, IRenderContext, TextureDesc, TextureError};

use crate::{
    CommandListIndex, Displayable, GpuResource, IGpu, IGpuCommandBuffer, IPipeline, Texture,
};

impl IPipeline for wgpu::RenderPipeline {}
impl IPipeline for wgpu::ComputePipeline {}
impl IBuffer for wgpu::Buffer {}
impl IBindGroup for wgpu::BindGroup {}

pub struct RenderContext<'a> {
    rpass: Rc<RefCell<wgpu::RenderPass<'a>>>
}

impl<'a> IContext<'a> for RenderContext<'a> {
    type Pipeline = wgpu::RenderPipeline;
    fn set_pipeline(&mut self, pipeline: &'a Self::Pipeline) {
        let rpass = self.rpass.get_mut();
        rpass.set_pipeline(pipeline);
    }
}

impl<'a> IRenderContext<'a>  for  RenderContext<'a>  {
    type Buffer = wgpu::Buffer;
    type BindGroup = wgpu::BindGroup;
    fn draw(&mut self) {
        todo!()
    }
    fn set_bind_group(&mut self,index: u32, group: &'a Self::BindGroup) {
                let rpass = self.rpass.get_mut();
        rpass.set_bind_group(index,group, &[]);
    }
    fn set_index_buffer(&mut self, buffer: &'a Self::Buffer) {
                        let rpass = self.rpass.get_mut();

        rpass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
    }
    fn set_vertex_buffer(&mut self, slot: u32,buffer: &'a Self::Buffer) {
        let rpass = self.rpass.get_mut();
        rpass.set_vertex_buffer(slot, buffer.slice(..));
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
    commands: BTreeMap<CommandListIndex, wgpu::CommandEncoder>,
}

impl<'a, T> IGpu<'a, T> for Gpu<'a, T>
where
    T: Displayable<'a>,
{
    type Texture = Texture;
    type RenderPipeline = wgpu::RenderPipeline;
    type RenderContext = RenderContext<'a>;
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
    fn submit_cmd_lists(&mut self) -> impl std::future::Future<Output = ()> + Send {
        async move {
            let keys: Vec<_> = self.commands.keys().map(|key| key.clone()).collect();
            let cmd_buffers = keys
                .iter()
                .map(|key| self.commands.remove(key).unwrap())
                .map(|encoder| encoder.finish());
            self.gpu_resource.queue.submit(cmd_buffers);
        }
    }
    fn begin_cmd_list(&mut self) -> CommandListIndex {
        let cmd_list_idx = CommandListIndex::new(self.node_id);
        let cmd_encoder =
            self.gpu_resource
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });
        self.commands.insert(cmd_list_idx, cmd_encoder);
        cmd_list_idx
    }
    fn begin_render_ctx(&mut self, idx: CommandListIndex) -> Result<Self::RenderContext, Self::Err> {
        let val = self.commands.get_mut(&idx);
        if let None = val {
            return Err(GpuError::NonExistantEncoder)
        }
        let val = val.unwrap();
        let rpass = val.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("Wgpu Render Pass"), color_attachments: &[], depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None });
        Ok(RenderContext { rpass: RefCell::new(rpass) })
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
    NonExistantEncoder
}
