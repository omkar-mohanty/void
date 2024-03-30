use uuid::Uuid;
use void_core::db::IId;

use crate::{Displayable, GpuResource};
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureId(Uuid);

impl IId for TextureId {}

pub struct TextureDesc {
    height: u32,
    width: u32,
}

pub trait ITexture<'a, T: Displayable<'a>>: Sized {
    type Err: Error;
    fn from_bytes(gpu_resource: &GpuResource<'a, T>, bytes: &[u8]) -> Result<Self, Self::Err>;
    fn create_depth_texture(gpu_resource: &GpuResource<'a, T>) -> Result<Self, Self::Err>;
}
