use na::Matrix4;
use winit::event::Event;

mod camera;
mod fps;
pub use camera::*;

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn with_aspect(width: f32, height: f32) -> Self {
        Self::new(width, height, 45.0, 0.1, 100.0)
    }
    pub fn new(width: f32, height: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width / height,
            fovy,
            znear,
            zfar,
        }
    }
    pub fn build_matrix(&self) -> na::Matrix4<f32> {
        *na::Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar).as_matrix()
    }
}

pub trait ICamera {
    fn build_view_matrix(&self) -> na::Matrix4<f32>;
    fn position(&self) -> na::Point3<f32>;
}

pub trait IController {
    fn input(&self, event: &Event<()>);
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view: [[f32; 4]; 4], // NEW!
    view_proj: [[f32; 4]; 4],
    inv_proj: [[f32; 4]; 4], // NEW!
    inv_view: [[f32; 4]; 4], // NEW!
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view: Matrix4::identity().into(),
            view_proj: Matrix4::identity().into(),
            inv_proj: Matrix4::identity().into(), // NEW!
            inv_view: Matrix4::identity().into(), // NEW!
        }
    }
    pub fn update_view_projection<T: ICamera>(&mut self, proj: &Projection, camera: &T) {
        self.view_position = camera.position().to_homogeneous().into();
        let proj = proj.build_matrix();
        let view = camera.build_view_matrix();
        let view_proj = proj * view;
        self.view = view.into();
        self.view_proj = view_proj.into();
        self.inv_proj = proj.try_inverse().unwrap().into();
        self.inv_view = view.transpose().into();
    }
}
