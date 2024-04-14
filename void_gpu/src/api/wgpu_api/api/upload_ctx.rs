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

impl<'a> IContext for UploadCtx<'a>
where
{
    type Out = CtxOut<'a>;
    fn new() -> Self {
        Self::default()
    }
    fn finish(self) -> Self::Out {
        CtxOut::Update(self)
    }
}

impl<'a> UpdateCamera<'a> for UploadCtx<'a>
where
{
    fn update_camera(&mut self, uniform: &'a [CameraUniform]) {
        self.upload_buffer(DEFAULT_CAMERA_BUFFER_ID, bytemuck::cast_slice(uniform));
    }
}

impl<'a> IUploadContext<'a> for UploadCtx<'a>
{
    fn upload_buffer(&mut self, buffer_id: BufferId, data: &'a [u8]) {
        self.data = Some(data);
        self.buffer_id = Some(buffer_id);
    }
}
