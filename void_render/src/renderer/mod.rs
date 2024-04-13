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
      let outs =  self.model_db.iter().map(|(_, model)| {
            let mut ctx = RenderCtx::new();
            ctx.draw_model(model, &self.camera);
            ctx.finish()
        });
        for out in outs {
            self.gpu.submit_ctx_out(out);
        }
        self.gpu.present().unwrap();
    }
}
