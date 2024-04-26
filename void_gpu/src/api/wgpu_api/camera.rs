use crate::{
    api::{BindGroupId, BufferId, DEFAULT_CAMERA_BIND_GROUP_ID, DEFAULT_CAMERA_BUFFER_ID},
    camera::ICamera,
};

pub struct Camera {
    eye: na::Point3<f32>,
    target: na::Point3<f32>,
    up: na::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    buffer_id: BufferId,
    group_id: BindGroupId,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            eye: na::Point3::new(0.0, 1.0, 2.0),
            target: na::Point3::new(0.0, 0.0, 0.0),
            up: na::Vector3::y(),
            aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            buffer_id: DEFAULT_CAMERA_BUFFER_ID,
            group_id: DEFAULT_CAMERA_BIND_GROUP_ID,
        }
    }
}

impl ICamera for Camera {
    fn build_view_projection_matrix(&self) -> na::Matrix4<f32> {
        let view = na::Matrix4::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = na::Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);
        proj * view
    }
    fn get_buffer(&self) -> BufferId {
        self.buffer_id
    }
    fn get_bind_group(&self) -> BindGroupId {
        self.group_id
    }
}
