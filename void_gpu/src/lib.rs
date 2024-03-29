mod api;
mod model;
mod texture;

pub use api::*;
pub use texture::{TextureId, TextureDesc};

pub(crate) struct Generic<T>(pub(crate) T);


