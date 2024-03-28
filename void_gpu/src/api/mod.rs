mod wgpu_api;
pub use wgpu_api::*;

pub struct Device<T> {
    device: T,
}

impl<T> Device<T> {
    pub fn create_context(&self) -> Context {
        todo!()
    }
}

pub struct Context {}
