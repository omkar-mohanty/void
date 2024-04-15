use uuid::Uuid;

use crate::api::wgpu_api::camera::Camera;
use crate::api::{Displayable, DrawModel, IBindGroup, IContext, IRenderContext, PipelineId};
use crate::model;

use super::{CtxOut, Gpu, RenderCmd};
use std::ops::Range;
use std::sync::Arc;

impl IBindGroup for wgpu::BindGroup {}

impl IContext for RenderCtx {
    type Out = CtxOut;

    fn finish(self) -> Self::Out {
        CtxOut::Render(self)
    }
}

impl<'a, 'b> DrawModel<'a> for RenderCtx
where
    'a: 'b,
{
    type Camera = Camera;

    fn draw_mesh(
        &mut self,
        mesh: &'a model::Mesh,
        material: &'a model::Material,
        camera_bind_group: &'a Self::Camera,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group)
    }
    fn draw_model(&mut self, model: &'a model::Model, camera_bind_group: &'a Self::Camera) {
        self.draw_model_instanced(model, 0..1, camera_bind_group)
    }
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a model::Mesh,
        material: &'a model::Material,
        instances: Range<u32>,
        camera_bind_group: &'a Self::Camera,
    ) {
        self.set_vertex_buffer(0, &mesh.vertex_buffer);
        self.set_index_buffer(1, &mesh.index_buffer);
        self.set_bind_group(0, &material.bind_group);
        self.set_bind_group(1, &camera_bind_group);
        self.draw(0..mesh.num_elements, 0, instances);
    }
    fn draw_model_instanced(
        &mut self,
        model: &'a model::Model,
        instances: Range<u32>,
        camera_bind_group: &'a Self::Camera,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group);
        }
    }
}

impl<'a> IRenderContext<'a> for RenderCtx {
    type BindGroup = wgpu::BindGroup;
    type Buffer = wgpu::Buffer;

    fn set_pipeline(&mut self, pipeline: PipelineId) {
        self.encode(move |cmd| {
            cmd.pipeline = Some(pipeline);
        });
    }

    fn set_bind_group(&mut self, slot: u32, group: &'a Self::BindGroup) {
        let mut render_cmds = self.gpu.context_manager.render_ctxs.write().unwrap();
        let cmd = render_cmds.get_mut(&self.id).unwrap();
        cmd.bind_groups.push((slot, group, &[]));
    }

    fn draw(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        let mut render_cmds = self.gpu.context_manager.render_ctxs.write().unwrap();
        let cmd = render_cmds.get_mut(&self.id).unwrap();
        cmd.draw_cmd = Some(DrawCmd {
            indices,
            base_vertex,
            instances,
        });
    }
    fn set_index_buffer(&mut self, _slot: u32, buffer: &'a Self::Buffer) {
        let mut render_cmds = self.gpu.context_manager.render_ctxs.write().unwrap();
        let cmd = render_cmds.get_mut(&self.id).unwrap();
        cmd.index_buffer = Some(buffer);
    }
    fn set_vertex_buffer(&mut self, slot: u32, buffer: &'a Self::Buffer) {
        let mut render_cmds = self.gpu.context_manager.render_ctxs.write().unwrap();
        let cmd = render_cmds.get_mut(&self.id).unwrap();
        cmd.vertex_buffer = Some((slot, buffer));
    }
}

pub struct RenderCtx {
    gpu: Arc<Gpu>,
    pub(crate) id: Uuid,
}

impl RenderCtx {
    pub fn new(gpu: Arc<Gpu>) -> Self {
        let mgr = &gpu.context_manager;
        let id = Uuid::new_v4();
        let mut ctxs = mgr.render_ctxs.write().unwrap();
        ctxs.insert(id, RenderCmd::default());
        drop(ctxs);
        Self { gpu, id }
    }

    fn encode<F: FnMut(&mut RenderCmd)>(&self, mut func: F) {
        let mut render_cmds = self.gpu.context_manager.render_ctxs.write().unwrap();
        let cmd = render_cmds.get_mut(&self.id).unwrap();
        func(cmd);
    }
}

#[derive(Default)]
pub(crate) struct DrawCmd {
    pub(super) indices: Range<u32>,
    pub(super) base_vertex: i32,
    pub(super) instances: Range<u32>,
}
