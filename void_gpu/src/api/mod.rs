mod wgpu_api;
use std::error::Error;
use std::ops::Range;
use std::sync::Arc;

use uuid::Uuid;

use void_core::IBuilder;
pub use wgpu_api::DataResource;
pub use wgpu_api::{Displayable, GpuResource, Texture, TextureError};
pub use wgpu_api::{Material, Mesh, Model};

use crate::texture::{ITexture, TextureDB};
use crate::{MaterialDB, MeshDB, ModelDB, TextureDesc};

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

pub trait IBuffer {}

pub trait IGpuCommandBuffer {}

pub trait IPipeline {}

pub trait IBindGroup {}

pub enum GpuPipeline {
    Render,
    Compute,
}

pub struct RenderPassDesc {}

pub struct ContextDesc {
    piptline_type: GpuPipeline,
}

pub trait IEncoder {
    type Buffer: IBuffer;
    type Pipeline: IPipeline;
    type BindGroup: IBindGroup;
}

pub trait IRenderEncoder<'a>: IEncoder {
    fn set_vertex_buffer(&mut self, slot: u32, buffer: &'a Self::Buffer);
    fn set_index_buffer(&mut self, buffer: &'a Self::Buffer);
    fn set_bind_group(&mut self, index: u32, group: &'a Self::BindGroup);
    fn set_pipeline(&mut self, pipeline: &'a Self::Pipeline);
    fn draw(&mut self, verts: Range<u32>, instances: Range<u32>);
}

pub trait IContext<'a, T: Displayable<'a>> {
    type CmdBuffer: IBuffer;
    type Encoder: IEncoder;

    fn new(gpu_resource: Arc<GpuResource<'a, T>>) -> Self;
    fn get_encoder<'b>(&'a self) -> Self::Encoder
    where
        'a: 'b;
    fn submit_encoders(&self, encoders: impl Iterator<Item = Self::Encoder>);
    fn end(&mut self) -> impl Iterator<Item = Self::CmdBuffer>;
}

pub trait IRenderContext<'a, D: Displayable<'a>, R: IRenderEncoder<'a>>:
    IContext<'a, D, Encoder = R>
{
    type Gpu: IGpu<'a, D>;
    type Err: Error;

    fn render(&mut self, gpu: &'a mut Self::Gpu) -> Result<(), Self::Err>;
    fn draw<'b>(&'b mut self, model_db: &'b ModelDB, mesh_db: &'b MeshDB, tex_db: &'b MaterialDB)
    where
        'b: 'a;
}

pub trait IUploadContext<'a, T: Displayable<'a>>: IContext<'a, T> {
    fn upload_buffer(buffer: &dyn IBuffer, data: impl bytemuck::Zeroable + bytemuck::Pod);
}

pub trait IGpu<'a, T: Displayable<'a>> {
    type Texture: ITexture<'a, T>;
    type CmdBuffer: IBuffer;
    type RenderPipeline: IPipeline;
    type ComputePipeline: IPipeline;
    type Err: std::error::Error;

    fn create_texture(&self, texture_desc: TextureDesc) -> Result<Self::Texture, Self::Err>;
    fn create_pipeline(
        &self,
        shader_src: &str,
        pipeline_builder: impl IBuilder<Output = Self::RenderPipeline>,
    ) -> Result<Self::RenderPipeline, Self::Err>;

    fn submit_cmds(&mut self, cmds: impl Iterator<Item = Self::CmdBuffer>);

    fn present(&mut self);
}

pub trait DrawModel<'a> {
    type BindGroup: IBindGroup;
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a Self::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a Self::BindGroup,
    );

    fn draw_model(&mut self, model: &'a Model, camera_bind_group: &'a Self::BindGroup);
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a Self::BindGroup,
    );
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
