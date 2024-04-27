use crate::{
    api::{camera::Camera, BufferId, IContext, IUploadContext},
    camera::{CameraUniform, ICamera, IUpdateCamera},
};

use super::CtxOut;

#[derive(Default)]
pub struct UploadCtx {
    pub(crate) buffer_id: Option<BufferId>,
    pub(crate) data: Option<Vec<u8>>,
}

impl IContext for UploadCtx {
    type Out = CtxOut;
    fn finish(self) -> Self::Out {
        CtxOut::Upload(self)
    }
}

impl<'a> IUpdateCamera<'a> for UploadCtx {
    type Camera = Camera;
    fn update_camera(&mut self, camera: &Self::Camera, uniform: &'a [CameraUniform]) {
        self.upload_buffer(camera.get_buffer(), bytemuck::cast_slice(uniform));
    }
}

impl<'a> IUploadContext<'a> for UploadCtx {
    fn upload_buffer(&mut self, buffer_id: BufferId, data: &'a [u8]) {
        self.buffer_id = Some(buffer_id);
        self.data = Some(data.to_vec())
    }
}
