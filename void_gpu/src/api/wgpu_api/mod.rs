pub(crate) mod api;
pub(crate) mod model;
pub(crate) mod pipeline;
pub(crate) mod texture;

use crate::model::{ModelVertex, Vertex};
use thiserror::Error;
use wgpu::{
    rwh::{HasDisplayHandle, HasWindowHandle},
    SurfaceTarget,
};

pub trait Displayable<'a>:
    Sync + Send + HasDisplayHandle + HasWindowHandle + Into<SurfaceTarget<'a>>
{
}

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("Surface Error {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
}

impl Vertex<wgpu::VertexBufferLayout<'static>> for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
