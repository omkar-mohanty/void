use std::ops::Deref;

use void_gpu::Displayable;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

pub use winit::*;


#[repr(transparent)]
pub struct Window(winit::window::Window);

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

impl<'a> Displayable<'a> for Window {
    // add code here
}
