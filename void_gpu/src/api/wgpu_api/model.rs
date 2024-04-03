use crate::{
    model::{MaterialId, MeshId},
    texture::TextureId,
};
use wgpu::{BindGroup, Buffer};

pub struct Material {
    pub(crate) name: String,
    pub(crate) id: MaterialId,
    pub(crate) diffuse_texture: TextureId,
    pub(crate) bind_group: BindGroup,
}

pub struct Mesh {
    pub(crate) name: String,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) num_elements: u32,
    pub(crate) material_id: MaterialId,
}

pub struct Model {
    pub(crate) meshes: Vec<MeshId>,
    pub(crate) materials: Vec<MaterialId>,
}
