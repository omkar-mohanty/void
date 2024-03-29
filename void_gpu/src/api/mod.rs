mod wgpu_api;
use uuid::Uuid;

pub use wgpu_api::{Texture, GpuResource, Displayable};

use crate::texture::TextureDesc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandListIndex(Uuid);

impl CommandListIndex {
    pub fn new(node_id: &[u8; 6]) -> Self {
        Self(Uuid::now_v6(&node_id))
    }
}

pub enum PipelineState {
    Compute,
    Render,
}

pub enum BufferType {
    Vertex,
    Index,
}

pub trait GpuApi {
    type Texture;
    type Buffer;
    type Model;

    fn create_texture(&self, desc: TextureDesc) -> Self::Texture;
    fn create_command_list(&mut self, ps: PipelineState) -> CommandListIndex;
    fn bind_buffer(&mut self, slot: u32, buffer: &Self::Buffer, cmd_index: CommandListIndex);
    fn submit_cmd_lists(&mut self);
}
