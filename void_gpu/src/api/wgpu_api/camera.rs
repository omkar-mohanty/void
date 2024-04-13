use crate::{
    api::{Gpu, DEFAULT_CAMERA_BUFFER_ID},
    camera::ICamera,
};
use std::{ops::Deref, sync::Arc};

use super::{pipeline::CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR, Displayable};
pub struct Camera {
    eye: na::Point3<f32>,
    target: na::Point3<f32>,
    up: na::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    group: wgpu::BindGroup,
}

impl Deref for Camera {
    type Target = wgpu::BindGroup;
    fn deref(&self) -> &Self::Target {
        &self.group
    }
}

impl Camera {
    pub fn new<'a, T: Displayable<'a>>(aspect: f32, gpu: Arc<Gpu<'a, T>>) -> Self {
        let layout = gpu
            .device
            .create_bind_group_layout(&CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR);
        let buffer_read = gpu.buffers.read().unwrap();
        let buffer = buffer_read.get(&DEFAULT_CAMERA_BUFFER_ID).unwrap();
        let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            eye: na::Point3::new(0.0, 1.0, 2.0),
            target: na::Point3::new(0.0, 0.0, 0.0),
            up: na::Vector3::y(),
            aspect,
            group,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

impl ICamera for Camera {
    type BindGroup = wgpu::BindGroup;
    fn build_view_projection_matrix(&self) -> na::Matrix4<f32> {
        let view = na::Matrix4::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = na::Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);
        proj * view
    }
}
