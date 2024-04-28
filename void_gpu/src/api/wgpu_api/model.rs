use crate::api::BindGroupId;
use crate::api::BufferId;
use crate::api::Texture;

#[allow(dead_code)]
pub struct Material {
    pub(crate) name: String,
    pub(crate) diffuse_texture: Texture,
    pub(crate) bind_group: BindGroupId,
}

#[allow(dead_code)]
pub struct Mesh {
    pub(crate) name: String,
    pub(crate) vertex_buffer: BufferId,
    pub(crate) instance_buffer: BufferId,
    pub(crate) index_buffer: BufferId,
    pub(crate) num_elements: u32,
    pub(crate) material: usize,
}

#[allow(dead_code)]
pub struct Model {
    pub(crate) meshes: Vec<Mesh>,
    pub(crate) materials: Vec<Material>,
}
