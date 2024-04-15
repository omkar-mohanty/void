use std::marker::PhantomData;

use crate::{
    api::{BufferId, IContext, IUploadContext},
    camera::{CameraUniform, UpdateCamera},
};

use super::{CtxOut, DEFAULT_CAMERA_BUFFER_ID};

#[derive(Default)]
pub struct UploadCtx<'a> {
    pub(crate) buffer_id: Option<BufferId>,
    pub(crate) data: Option<&'a [u8]>,
    pub(crate) _phantom: PhantomData<&'a ()>,
}
