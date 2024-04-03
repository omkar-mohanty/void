use crate::{
    api::wgpu_api::model::{Material, Mesh, Model},
    ResourceDB,
};
use uuid::Uuid;
use void_core::db::IId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialId(Uuid);

impl IId for MaterialId {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshId(Uuid);

impl IId for MeshId {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelId(Uuid);

impl IId for ModelId {}

pub type MeshDB = ResourceDB<MeshId, Mesh>;
pub type MaterialDB = ResourceDB<MaterialId, Material>;
pub type ModelDB = ResourceDB<ModelId, Model>;

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
