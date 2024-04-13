use std::marker::PhantomData;

use rand::distributions::uniform;

use crate::{
    api::{BufferId, IContext, IUploadContext},
    camera::{CameraUniform, UpdateCamera},
};

use super::{CtxOut, DEFAULT_CAMERA_BUFFER_ID};

#[derive(Default)]
pub struct UploadCtx<'a, 'b> {
    pub(crate) buffer_id: Option<BufferId>,
    pub(crate) data: Option<&'b [u8]>,
    pub(crate) _phantom: PhantomData<&'a ()>,
}

impl<'a, 'b> IContext<'a, 'b> for UploadCtx<'a, 'b>
where
    'b: 'a,
{
    type Out = CtxOut<'a, 'b>;
    fn new() -> Self {
        Self::default()
    }
    fn finish(self) -> Self::Out {
        CtxOut::Update(self)
    }
}

impl<'a, 'b> UpdateCamera<'a, 'b> for UploadCtx<'a, 'b>
where
    'b: 'a,
{
    fn update_camera(&mut self, uniform: &'b [CameraUniform]) {
        self.upload_buffer(DEFAULT_CAMERA_BUFFER_ID, bytemuck::cast_slice(uniform));
    }
}

impl<'a, 'b> IUploadContext<'a, 'b> for UploadCtx<'a, 'b>
where
    'b: 'a,
{
    fn upload_buffer(&mut self, buffer_id: BufferId, data: &'b [u8]) {
        self.data = Some(data);
        self.buffer_id = Some(buffer_id);
    }
}
