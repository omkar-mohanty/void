use nalgebra as na;

use winit::{
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use super::ICamera;

pub struct StaticCamera {
    pub position: na::Point3<f32>,
    pub target: na::Point3<f32>,
    pub up: na::Vector3<f32>,
}

impl StaticCamera {
    pub fn new() -> Self {
        StaticCamera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            position: na::Point3::new(0.0, 1.0, 2.0),
            // have it look at the origin
            target: na::Point3::origin(),
            // which way is "up"
            up: *na::Vector3::y_axis(),
        }
    }
}

impl ICamera for StaticCamera {
    fn position(&self) -> na::Point3<f32> {
        self.position
    }
    fn build_view_matrix(&self) -> na::Matrix4<f32> {
        na::Isometry3::look_at_rh(&self.position, &self.target, &self.up).to_matrix()
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

    pub fn update_camera(&self, camera: &mut StaticCamera) {
        let forward = camera.target - camera.position;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.position += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.position -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(&camera.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target - camera.position;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.position =
                camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.position =
                camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
