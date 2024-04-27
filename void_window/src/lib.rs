use std::ops::Deref;
use void_gpu::api::IDisplayable;

use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

pub use winit::*;

#[repr(transparent)]
pub struct Window(pub winit::window::Window);

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<wgpu::rwh::WindowHandle<'_>, wgpu::rwh::HandleError> {
        self.0.window_handle()
    }
}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<wgpu::rwh::DisplayHandle<'_>, wgpu::rwh::HandleError> {
        self.0.display_handle()
    }
}

impl Deref for Window {
    type Target = winit::window::Window;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IDisplayable for Window {
    fn request_redraw(&self) {
        self.0.request_redraw();
    }
    fn width(&self) -> f32 {
        self.0.inner_size().width as f32
    }
    fn height(&self) -> f32 {
        self.0.inner_size().height as f32
    }
}
