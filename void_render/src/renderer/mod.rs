use std::sync::Arc;
use thiserror::Error;
use void_core::db::IDb;
use void_gpu::{
    api::{render_ctx::RenderCtx, DrawModel, Gpu, GpuError, IContext, IGpu},
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

    pub fn update(&mut self) {
        self.camera_uniform.update_view_proj(&self.camera);
    }

    pub fn add_model(&mut self, model: Model) {
        self.model_db.insert(std::iter::once(model));
    }

    pub fn render(&self) -> Result<(), RenderError> {
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
        self.gpu.present()?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Error creating texture {0}")]
    GpuError(#[from] GpuError),
}
