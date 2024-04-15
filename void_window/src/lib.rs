use std::ops::Deref;

use void_gpu::api::Displayable;
use wgpu::{rwh::{HasDisplayHandle, HasWindowHandle}, SurfaceTarget};

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

impl Into<SurfaceTarget> for Window {
    fn into(self) -> SurfaceTarget {
        SurfaceTarget::Window(window)
    }
}

impl<'a> Displayable<'a> for Window {
    // add code here
}
