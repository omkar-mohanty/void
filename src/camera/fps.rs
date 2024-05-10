use std::sync::{Arc, RwLock};
use std::{f64::consts::FRAC_PI_2, time::Duration};

use winit::event::{Event, MouseButton};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use super::{ICamera, IController};

const SAFE_FRAC_PI_2: f64 = FRAC_PI_2 - 0.001;

pub struct FpsCamera {
    positon: na::Point3<f64>,
    yaw: f64,
    pitch: f64,
}

pub struct FpsController {
    amount_left: f64,
    amount_right: f64,
    amount_forward: f64,
    amount_backward: f64,
    amount_up: f64,
    amount_down: f64,
    rotate_horizontal: f64,
    rotate_vertical: f64,
    scroll: f64,
    speed: f64,
    sensitivity: f64,
}

impl Default for FpsController {
    fn default() -> Self {
        Self::new(4.0, 0.4)
    }
}

impl FpsController {
    pub fn new(speed: f64, sensitivity: f64) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        use KeyCode::*;
        let amount = if state.is_pressed() { 1.0 } else { 0.0 };
        match key {
            KeyA => {
                self.amount_left = amount;
                true
            }
            KeyD => {
                self.amount_right = amount;
                true
            }
            KeyW => {
                self.amount_forward = amount;
                true
            }
            KeyS => {
                self.amount_backward = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx;
        self.rotate_vertical = mouse_dy;
    }

    pub fn process_scroll(&mut self, mouse_delta: &MouseScrollDelta) {
        use MouseScrollDelta::*;
        self.scroll = -match mouse_delta {
            LineDelta(_, scroll) => (scroll * 100.0).into(),
            PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll,
        }
    }

    pub fn update_camera(&mut self, camera: &mut FpsCamera, dt: Duration) {
        let dt = dt.as_secs_f64();
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = na::Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = na::Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.positon += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.positon += right * (self.amount_right - self.amount_left) * self.speed * dt;

        let (pitch_cos, pitch_sin) = camera.pitch.sin_cos();
        let scrollward = na::Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin);
        camera.positon += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        camera.positon.y = (self.amount_up - self.amount_down) * self.speed * dt;

        camera.yaw += self.rotate_horizontal * self.sensitivity * dt;
        camera.pitch += -self.rotate_vertical * self.sensitivity * dt;

        self.rotate_vertical = 0.0;
        self.rotate_horizontal = 0.0;

        if camera.pitch < SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        use WindowEvent::*;

        match event {
            KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                self.process_keyboard(*key_code, *state);
            }
            MouseWheel { delta, .. } => {
                self.process_scroll(delta);
            }
            _ => {}
        }
    }
}

impl IController for Arc<RwLock<FpsController>> {
    fn input(&self, event: &Event<()>) {
        use winit::event::DeviceEvent::*;
        use Event::*;
        let mut controller = self.write().unwrap();

        match event {
            DeviceEvent { event, .. } => match event {
                MouseMotion { delta } => {
                    controller.process_mouse(delta.0, delta.1);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
