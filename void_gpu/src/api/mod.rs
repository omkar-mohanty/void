pub(crate) mod wgpu_api;
use std::ops::Range;
use std::sync::Arc;
use std::{error::Error, future::Future};

use uuid::Uuid;

use crate::{
    model::{MaterialDB, MeshDB, ModelDB},
    TextureDesc,
};
use void_core::IBuilder;
use wgpu_api::model::{Material, Mesh, Model};
pub use wgpu_api::{api::*, texture::Texture, texture::TextureError, Displayable};

use crate::texture::ITexture;

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

#[derive(Clone, Copy, Debug)]
pub enum CtxType {
    Render,
    Compute,
}

pub trait IContext<'a, T: Displayable<'a>> {
    type CmdBuffer: IBuffer;
    type Encoder: IEncoder;

    fn get_encoder<'b>(&'a self) -> Self::Encoder
    where
        'a: 'b;
    fn ctx_type(&self) -> CtxType;
    fn end(self) -> impl Iterator<Item = Self::CmdBuffer>;
}

pub trait IRenderContext<'a, D: Displayable<'a>, R: IRenderEncoder<'a>>:
    IContext<'a, D, Encoder = R>
{
    type Err: Error;

    fn render(&self) -> Result<(), Self::Err>;
    fn set_depth_texture(&mut self, texture: Texture);
}

pub trait IUploadContext<'a, T: Displayable<'a>>: IContext<'a, T> {
    fn upload_buffer(buffer: &dyn IBuffer, data: impl bytemuck::Zeroable + bytemuck::Pod);
}

pub trait IGpu<'a, T: Displayable<'a>> {
    type CmdBuffer: IBuffer;
    type Encoder: IEncoder;
    type RenderPipeline: IPipeline;
    type ComputePipeline: IPipeline;
    type Err: std::error::Error;

    fn record_recurring_cmd<'b>(&'a mut self, depth_tex: Texture, func: impl FnMut(&mut Self::Encoder));

    fn submit_ctx(&mut self, render_ctx: impl IContext<'a, T, CmdBuffer = Self::CmdBuffer>);

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
