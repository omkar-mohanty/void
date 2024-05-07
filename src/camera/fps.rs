use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseScrollDelta},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct FpsCamera {
    positon: na::Point3<f32>,
    yaw: f32,
    pitch: f32,
}

pub struct FpsController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl Default for FpsController {
    fn default() -> Self {
        Self::new(4.0, 0.4)
    }
}

impl FpsController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
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

    pub fn process_mouse(&mut self, mouse_dx: f32, mouse_dy: f32) {
        self.rotate_horizontal = mouse_dx;
        self.rotate_vertical = mouse_dy;
    }

    pub fn process_scroll(&mut self, mouse_delta: &MouseScrollDelta) {
        use MouseScrollDelta::*;
        self.scroll = -match mouse_delta {
            LineDelta(_, scroll) => scroll * 100.0,
            PixelDelta(PhysicalPosition {  y: scroll, .. }) => *scroll as f32
        }
    }
}
