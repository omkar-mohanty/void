use std::{iter, sync::Arc};
use void_gpu::{
    api::{Displayable, DrawModel, Gpu, IContext, IGpu, StaticRenderCtx},
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
        let render_ctx = StaticRenderCtx::new(Arc::clone(&self.gpu));
        render_ctx.draw_model_nbd(model);
        let ctx_out = render_ctx.end();
        self.gpu.submit_ctx_output(iter::once(ctx_out));
    }

    pub fn render(&self) {
        self.gpu.present().unwrap();
    }
}
