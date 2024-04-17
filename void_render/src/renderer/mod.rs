use std::sync::Arc;
use void_core::db::IDb;
use void_gpu::{
    api::{render_ctx::RenderCtx, DrawModel, Gpu, IContext, IGpu},
    camera::{Camera, CameraUniform},
    model::*,
};

pub struct RendererEngine {
    gpu: Arc<Gpu>,
    camera: Camera,
    camera_uniform: CameraUniform,
    model_db: ModelDB,
}

impl RendererEngine {
    pub fn new(gpu: Arc<Gpu>, aspect: f32) -> Self {
        let camera = Camera::new(aspect);
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
        let _outs: Vec<_> = self
            .model_db
            .iter()
            .map(|(_, model)| {
                let mut ctx = RenderCtx::new();
                ctx.draw_model(model, &self.camera);
                let out = ctx.finish();
                self.gpu.submit_ctx_out(out);
            })
            .collect();
        self.gpu.present().unwrap();
    }
}
