pub(crate) mod wgpu_api;
use std::ops::Range;

use uuid::Uuid;

use wgpu_api::model::{Material, Mesh, Model};
pub use wgpu_api::{
    api::*, pipeline::PipelineBuilder, texture::Texture, texture::TextureError, Displayable,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub struct PipelineId(Uuid);

pub trait IBuffer {}

pub trait IGpuCommandBuffer {}

pub trait IPipeline {}

pub trait IBindGroup {}

pub enum GpuPipeline {
    Render(wgpu::RenderPipeline),
    Compute(wgpu::ComputePipeline),
}

pub struct RenderPassDesc {}

pub trait IContext {
    type Output;
    type Encoder;

    fn set_pileine(&mut self, id: PipelineId);
    fn set_stage(&self, enc: Self::Encoder);
    fn end(self) -> Self::Output;
}

pub trait IGpu {
    type CtxOutput;
    type Err: std::error::Error;

    fn submit_ctx_output(&self, render_ctx: impl Iterator<Item = Self::CtxOutput>);
    fn insert_pipeline(&self, pipeline: GpuPipeline) -> PipelineId;
    fn window_update(&self, width: u32, height: u32);
    fn present(&self) -> Result<(), Self::Err>;
}

pub trait DrawModel<'a, 'b> {
    type BindGroup: IBindGroup;
    fn draw_mesh(
        &self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b Self::BindGroup,
    );
    fn draw_mesh_instanced(
        &self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        camera_bind_group: &'b Self::BindGroup,
    );

    fn draw_model(&self, model: &'b Model, camera_bind_group: &'b Self::BindGroup);
    fn draw_model_instanced(
        &self,
        model: &'b Model,
        instances: Range<u32>,
        camera_bind_group: &'b Self::BindGroup,
    );
    fn draw_mesh_nbd(&self, mesh: &'b Mesh, material: &'b Material);
    fn draw_mesh_nbd_instanced(
        &self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
    );
    fn draw_model_nbd(&self, model: &'b Model);
    fn draw_model_nbd_instanced(&self, model: &'b Model, instances: Range<u32>);
}

#[cfg(test)]
mod tests {
    use std::{
        thread::{self, sleep},
        time::Duration,
    };

    use rand::Rng;

    use crate::api::CommandListIndex;

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
