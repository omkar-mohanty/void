pub use crate::api::wgpu_api::model::{Material, Mesh, Model};
use crate::ResourceDB;
use na::Vector3;
use uuid::Uuid;
use void_core::db::IId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ID(Uuid);

impl IId for ID {
    fn new() -> Self {
        ID(Uuid::new_v4())
    }
}
pub type ModelDB = ResourceDB<ID, ModelEntry>;

pub struct ModelEntry {
    pub instances: Vec<Instance>,
    pub model: Model,
}

pub(crate) trait Vertex<T> {
    fn desc() -> T;
}

pub struct Instance {
    pub(crate) position: na::Vector3<f32>,
    pub(crate) rotation: na::UnitQuaternion<f32>,
}

impl Default for Instance {
    fn default() -> Self {
        let position = na::Vector3::new(0.0, 0.0, 0.0);
        let rotation = na::UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.0);
        Instance { position, rotation }
    }
}

impl Instance {
    pub(crate) fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (na::Matrix4::new_translation(&self.position)
                * na::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub(crate) model: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub(crate) struct ModelVertex {
    pub(crate) position: [f32; 3],
    pub(crate) tex_coords: [f32; 2],
    pub(crate) normal: [f32; 3],
}
