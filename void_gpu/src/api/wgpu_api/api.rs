use std::collections::BTreeMap;

use crate::{api::{BufferType, CommandListIndex, GpuApi}, model::Vertex};

use super::GpuResource;
use wgpu::{
    rwh::{HasDisplayHandle, HasWindowHandle},
    util::DeviceExt,
    SurfaceTarget,
};

pub struct GpuApiImpl<'a, T>
where
    T: Sync + Send + HasDisplayHandle + HasWindowHandle + Into<SurfaceTarget<'a>>,
{
    gpu_resource: GpuResource<'a, T>,
    gpu_commands: BTreeMap<CommandListIndex, wgpu::CommandEncoder>,
}

impl<'a, T> GpuApi for GpuApiImpl<'a, T>
where
    T: Sync + Send + HasDisplayHandle + HasWindowHandle + Into<SurfaceTarget<'a>>,
{
    fn bind_buffer(
        &mut self,
        slot: u32,
        buffer: &wgpu::Buffer,
        cmd_index: CommandListIndex,
        buffer_type: crate::api::BufferType,
    ) {
        let encoder = self.gpu_commands.get_mut(&cmd_index);

        if encoder.is_none() {
            return;
        }

        let encoder = encoder.unwrap();
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass Descriptor"),
                color_attachments: &[],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            match buffer_type {
                BufferType::Vertex => rpass.set_vertex_buffer(slot, buffer.slice(..)),
                BufferType::Index => rpass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16)
            }
        }
    }

    fn create_texture(&self, desc: crate::texture::TextureDesc) -> crate::texture::Texture {
        todo!()
    }

    fn submit_cmd_lists(&mut self) {
        todo!()
    }

    fn create_command_list(&mut self, ps: crate::api::PipelineState) -> CommandListIndex {
        todo!()
    }
}
