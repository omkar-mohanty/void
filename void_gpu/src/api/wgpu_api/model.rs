use crate::api::BindGroupId;
use crate::api::BufferId;
use crate::api::Texture;

pub struct Material {
    pub(crate) name: String,
    pub(crate) diffuse_texture: Texture,
    pub(crate) bind_group: BindGroupId,
}

pub struct Mesh {
    pub(crate) name: String,
    pub(crate) vertex_buffer: BufferId,
    pub(crate) index_buffer: BufferId,
    pub(crate) num_elements: u32,
    pub(crate) material: usize,
}

pub struct Model {
    pub(crate) meshes: Vec<Mesh>,
    pub(crate) materials: Vec<Material>,
}
