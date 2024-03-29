mod wgpu_api;
use rand::Rng;
use uuid::Uuid;
pub use wgpu_api::*;

use crate::texture::{Texture, TextureDesc};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandListIndex(Uuid);

impl CommandListIndex {
    pub fn new() -> Self {
        let node_id = rand::thread_rng().gen::<[u8; 6]>();
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
    fn create_texture(&self, desc: TextureDesc) -> Texture;
    fn create_command_list(&mut self, ps: PipelineState) -> CommandListIndex;
    fn bind_buffer(
        &mut self,
        slot: u32,
        buffer: &Buffer,
        cmd_index: CommandListIndex,
        buffer_type: BufferType,
    );
    fn submit_cmd_lists(&mut self);
}
