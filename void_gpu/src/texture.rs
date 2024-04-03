use uuid::Uuid;
use void_core::db::IId;

use crate::{
    api::{Displayable, GpuResource},
    ResourceDB,
};
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureId(Uuid);

impl IId for TextureId {}

pub type TextureDB<'a, T> = ResourceDB<TextureId, T>;

pub struct TextureDesc<'a> {
    pub(crate) height: u32,
    pub(crate) width: u32,
    pub(crate) data: &'a [u8],
}

pub trait ITexture<'a, T: Displayable<'a>>: Sized {
    type Err: Error;
    fn from_bytes(gpu_resource: &GpuResource<'a, T>, bytes: &[u8]) -> Result<Self, Self::Err>;
    fn create_depth_texture(gpu_resource: &GpuResource<'a, T>) -> Result<Self, Self::Err>;
}
