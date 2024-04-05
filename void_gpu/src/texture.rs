use crate::api::{Displayable, Gpu};
use std::error::Error;

pub trait ITexture<'a, T: Displayable<'a>>: Sized {
    type Err: Error;
    fn from_bytes(gpu_resource: &Gpu<'a, T>, bytes: &[u8]) -> Result<Self, Self::Err>;
    fn create_depth_texture(gpu_resource: &Gpu<'a, T>) -> Result<Self, Self::Err>;
}
