use std::sync::Arc;
use void_core::db::IDb;
use void_gpu::{
    api::{render_ctx::RenderCtx, Displayable, DrawModel, Gpu, IContext, IGpu},
    camera::{Camera, CameraUniform},
    model::*,
};

pub struct RendererEngine<'a, T: Displayable<'a>> {
    gpu: Arc<Gpu<'a, T>>,
    camera: Camera,
    camera_uniform: CameraUniform,
    model_db: ModelDB,
}

impl<'a, T: Displayable<'a> + 'a> RendererEngine<'a, T> {
    pub fn new(gpu: Arc<Gpu<'a, T>>, aspect: f32) -> Self {
        let camera = Camera::new(aspect, Arc::clone(&gpu));
        Self {
            gpu,
            camera,
            model_db: ModelDB::default(),
            camera_uniform: CameraUniform::new(),
        }
    }

    pub fn add_model(&mut self, model: Model) {
        self.model_db.insert(std::iter::once(model));
    }

    pub fn render(&self) {
        let outs: Vec<_> = self
            .model_db
            .iter()
            .map(|(_, model)| {
                let mut ctx = RenderCtx::new(Arc::clone(&self.gpu));
                ctx.draw_model(model, &self.camera);
                let out = ctx.finish();
                self.gpu.submit_ctx_out(out);
            })
            .collect();
        self.gpu.present().unwrap();
    }
}
