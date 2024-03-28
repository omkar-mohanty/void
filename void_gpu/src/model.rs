use crate::api::{BindGroup, Buffer};
use crate::texture;
use std::ops::Range;
use void_core::db::{IDb, IId};

pub trait Vertex<T> {
    fn desc() -> T;
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub(crate) struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub bind_group: BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub struct ModelDB {
    models: Vec<Model>,
}

pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a BindGroup,
    );

    fn draw_model(&mut self, model: &'a Model, camera_bind_group: &'a BindGroup);
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a BindGroup,
    );
}

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct ModelID(usize);

impl IId for ModelID {}

impl IDb for ModelDB {
    type Id = ModelID;
    type Data = Model;
    fn get(
        &self,
        _ids: impl Iterator<Item = Self::Id>,
    ) -> Result<impl Iterator<Item = &Self::Data>, void_core::db::DbError<Self::Id>> {
        Ok(self.models.iter())
    }
}
