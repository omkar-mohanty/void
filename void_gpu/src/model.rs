pub use crate::api::wgpu_api::model::{Material, Mesh, Model};
use crate::ResourceDB;
use uuid::Uuid;
use void_core::db::IId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ID(Uuid);

impl IId for ID {
    fn new() -> Self {
        ID(Uuid::new_v4())
    }
}
pub type ModelDB = ResourceDB<ID, Model>;

pub(crate) trait Vertex<T> {
    fn desc() -> T;
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub(crate) struct ModelVertex {
    pub(crate) position: [f32; 3],
    pub(crate) tex_coords: [f32; 2],
    pub(crate) normal: [f32; 3],
}
