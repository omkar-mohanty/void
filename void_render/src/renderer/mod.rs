use std::{iter, sync::Arc};
use void_gpu::{
    api::{render_ctx::RenderCtx, Displayable, DrawModel, Gpu, IContext, IGpu},
    model::*,
};

pub struct RendererEngine<'a, T: Displayable<'a>> {
    gpu: Arc<Gpu<'a, T>>,
}

impl<'a, T: Displayable<'a> + 'a> RendererEngine<'a, T> {
    pub fn new(gpu: Arc<Gpu<'a, T>>) -> Self {
        Self { gpu }
    }

    pub fn add_model(&self, model: &'a Model) {
        todo!("Add model adding logic");
    }

    pub fn render(&self) {
        self.gpu.present().unwrap();
    }
}
