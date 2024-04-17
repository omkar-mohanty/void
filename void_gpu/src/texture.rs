use crate::api::Gpu;
use std::error::Error;

pub trait ITexture: Sized {
    type Err: Error;
    fn from_bytes(gpu_resource: &Gpu, bytes: &[u8]) -> Result<Self, Self::Err>;
    fn create_depth_texture(gpu_resource: &Gpu) -> Result<Self, Self::Err>;
}
