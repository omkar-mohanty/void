use std::ops::Deref;

pub use crate::api::camera::Camera;

pub trait ICamera: Deref<Target = Self::BindGroup> {
    type BindGroup;

    fn build_view_projection_matrix(&self) -> na::Matrix4<f32>;
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: na::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj<T: ICamera>(&mut self, camera: &T) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub trait UpdateCamera<'a, 'b>
where
    'b: 'a,
{
    fn update_camera(&mut self, uniform: &'b [CameraUniform]);
}
