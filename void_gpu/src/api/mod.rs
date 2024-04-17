pub(crate) mod wgpu_api;
use std::ops::Range;

use uuid::Uuid;

use void_core::{rayon::iter::ParallelIterator, IBuilder};
use wgpu_api::model::{Material, Mesh, Model};
pub use wgpu_api::{
    api::*, camera, pipeline::PipelineBuilder, texture::Texture, texture::TextureError,
    IDisplayable,
};

use crate::camera::{ICamera, UpdateCamera};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct CommandListIndex(Uuid);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct BufferId(Uuid);

pub trait IBuffer {}

pub trait IBindGroup {}

pub trait IPipeline {}

pub trait ICtxOut: Send + Sync {}

pub trait IContext {
    type Out;
    fn finish(self) -> Self::Out;
}

pub trait IRenderContext<'a>: IContext + DrawModel<'a> {
    fn set_pipeline(&mut self, pipeline: PipelineId);
    fn set_bind_group(&mut self, slot: u32, bind_group: usize);
    fn set_vertex_buffer(&mut self, slot: u32, buffer: BufferId);
    fn set_index_buffer(&mut self, slot: u32, buffer: BufferId);
    fn draw(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>);
}

pub trait IUploadContext<'a>: IContext + UpdateCamera<'a> {
    fn upload_buffer(&mut self, buffer_id: BufferId, data: &'a [u8]);
}

pub trait IComputeContext<'a, 'b>: IContext
where
    'b: 'a,
{
}

impl CommandListIndex {
    pub fn new(node_id: &[u8; 6]) -> Self {
        Self(Uuid::now_v6(&node_id))
    }
}

/// Public facing pipeline type. Meant for clints to specify pipeline type.
pub enum PipelineType {
    Render,
    Compute,
}

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub struct PipelineId(Uuid);

pub trait IGpu {
    type Err: std::error::Error;
    type CtxOut: ICtxOut;

    fn submit_ctx_out(&self, out: Self::CtxOut);
    fn submit_ctx_bundled(&self, outs: impl ParallelIterator<Item = Self::CtxOut>)
    where
        Self: Sync,
    {
        outs.for_each(|out| {
            self.submit_ctx_out(out);
        });
    }
    fn create_buffer(&self) -> BufferId;
    fn window_update(&self, width: u32, height: u32);
    fn present(&self) -> Result<(), Self::Err>;
}

pub trait DrawModel<'b> {
    type Camera: ICamera;
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b Self::Camera,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        camera_bind_group: &'b Self::Camera,
    );

    fn draw_model(&mut self, model: &'b Model, camera_bind_group: &'b Self::Camera);
    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        camera_bind_group: &'b Self::Camera,
    );
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
