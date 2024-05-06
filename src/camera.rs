use nalgebra as na;

use crate::gpu::Gpu;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{
        Key::{self, Character}, KeyCode, PhysicalKey
    },
};

pub struct Camera {
    pub eye: na::Point3<f32>,
    pub target: na::Point3<f32>,
    pub up: na::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(gpu: &Gpu) -> Self {
        let (height, width) =
            gpu.get_config_read(|config| (config.height as f32, config.width as f32));
        Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: na::Point3::new(0.0, 1.0, 2.0),
            // have it look at the origin
            target: na::Point3::origin(),
            // which way is "up"
            up: *na::Vector3::y_axis(),
            aspect: width / height,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

impl Camera {
    fn build_view_projection_matrix(&self) -> na::Matrix4<f32> {
        // 1.
        let view = na::Isometry3::look_at_rh(&self.eye, &self.target, &self.up);
        // 2.
        let proj = na::Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar);
        // 3.
        proj.as_matrix() * view.to_matrix()
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
    view_position: [f32; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: na::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
        self.view_position = camera.eye.to_homogeneous().into();
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new(0.2)
    }
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_key(&mut self, key_event: &KeyEvent) {
        if let KeyEvent {
            state,
            physical_key: PhysicalKey::Code(key_code),
            ..
        } = key_event
        {
            self.process_events(&key_code, state.is_pressed());
        }
    }

    pub fn process_events(&mut self, key: &KeyCode, pressed: bool) {
        use KeyCode::*;
        match key {
            KeyW => {
                self.is_forward_pressed = pressed;
            }
            KeyA => {
                self.is_left_pressed = pressed;
            }
            KeyS => {
                self.is_backward_pressed = pressed;
            }
            KeyD => {
                self.is_right_pressed = pressed;
            }
            _ => {}
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(&camera.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
