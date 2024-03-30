mod wgpu_api;
use std::future::Future;
use std::ops::Range;

use uuid::Uuid;

pub use wgpu_api::DataResource;
pub use wgpu_api::{Displayable, GpuResource, Texture, TextureError};
pub use wgpu_api::{Material, Mesh, Model};

use crate::texture::ITexture;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandListIndex(Uuid);

impl CommandListIndex {
    pub fn new(node_id: &[u8; 6]) -> Self {
        Self(Uuid::now_v6(&node_id))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BufferType {
    Vertex,
    Uniform,
}

#[derive(Clone, Copy, Debug)]
pub struct BufferDesc {
    pub size: usize,
    pub usage: BufferType,
    pub stride: usize,
}

pub trait IBuffer: Sized {
    fn desc(&self) -> BufferDesc;
    fn new(desc: BufferDesc) -> Self;
}

pub trait IGpuCommandBuffer {}

pub trait IPipeline {}

pub trait IContext {
    type CmdBuffer: IGpuCommandBuffer;
    fn end(self) -> Self::CmdBuffer;
}

pub trait IGraphicsContext: IContext {
    type Buffer: IBuffer;
    type Pipeline: IPipeline;

    fn set_vertex_buffer(&mut self, buffer: &Self::Buffer);
    fn set_index_buffer(&mut self, buffer: &Self::Buffer);
    fn set_pipeline(&mut self, pipeline: &Self::Pipeline);
    fn draw(&mut self);
    fn draw_instanced(&mut self, range: Range<u32>);
}

pub trait IComputeContext: IContext {
    type Pipeline: IPipeline;

    fn set_pipeline(&mut self, pipeline: &Self::Pipeline);
    fn dispatch(&mut self);
}

pub trait IUploadContext<'a, T>: IContext
where
    T: Displayable<'a>,
{
    type Buffer: IBuffer;
    type Texture: ITexture<'a, T>;

    fn upload_buffer(&mut self, buffer: &Self::Buffer);
    fn upload_texture(&mut self, texture: &Self::Texture);
}

pub trait IGpu<'a, T: Displayable<'a>> {
    type Texture: ITexture<'a, T>;
    type Buffer: IBuffer;
    type Pipeline: IPipeline;
    type CmdBuffer: IGpuCommandBuffer;

    fn create_buffer(&self) -> Self::Buffer;
    fn create_texture(&self) -> Self::Texture;
    fn create_pipeline(&self) -> Self::Pipeline;

    fn submit_context(&self, context: impl IContext) -> impl Future<Output = ()> + Send;
    fn present(&self);
}

#[cfg(test)]
mod tests {
    use std::{
        thread::{self, sleep},
        time::Duration,
    };

    use rand::Rng;

    use crate::CommandListIndex;

    #[test]
    fn test_cmd_list() {
        let node_id = rand::thread_rng().gen::<[u8; 6]>();
        let v1 = CommandListIndex::new(&node_id);
        let v2 = CommandListIndex::new(&node_id);

        assert!(v1 < v2);
    }

    #[test]
    fn test_cmd_list_thread() {
        let node_id = rand::thread_rng().gen::<[u8; 6]>();

        let v1 = thread::scope(|_| CommandListIndex::new(&node_id));
        let v2 = thread::scope(|_| CommandListIndex::new(&node_id));

        assert!(v1 < v2);
    }

    #[test]
    fn test_cmd_list_wait() {
        let node_id = rand::thread_rng().gen::<[u8; 6]>();

        let v1 = thread::scope(|_| CommandListIndex::new(&node_id));
        let v2 = thread::scope(|_| {
            sleep(Duration::from_millis(1));
            CommandListIndex::new(&node_id)
        });

        assert!(v1 < v2);
    }
}
