use uuid::Uuid;
use void_core::db::IId;
use wgpu::{BindGroup, Buffer};
use crate::{Generic, TextureId};

pub type Material = Generic<MaterialInner>;
pub type Mesh = Generic<MeshInner>;
pub type Model = Generic<ModelInner>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialId(Uuid);

impl IId for MaterialId {
    
}

pub struct MaterialInner {
    pub name: String,
    pub id: MaterialId,
    pub diffuse_texture: TextureId,
    pub bind_group: BindGroup,
}

pub struct MeshInner {
    pub name: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_elements: u32,
    pub material_id: MaterialId,
}

pub struct ModelInner {
    pub meshes: Vec<MeshInner>,
    pub materials: Vec<MaterialInner>,
}
