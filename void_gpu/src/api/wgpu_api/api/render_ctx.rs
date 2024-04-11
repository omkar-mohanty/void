use wgpu::DynamicOffset;

use crate::api::{DrawModel, IBindGroup, IContext, IRenderContext};
use crate::model;

use super::CtxOut;
use std::marker::PhantomData;
use std::ops::Range;

impl IBindGroup for wgpu::BindGroup {}

impl<'a, 'b> IContext<'a, 'b> for RenderCtx<'a, 'b>
where
    'b: 'a,
{
    type Pipeline = wgpu::RenderPipeline;
    type BindGroup = wgpu::BindGroup;
    type Out = CtxOut<'a, 'b>;

    fn new() -> Self {
        Self::default()
    }

    fn set_pipeline(&mut self, pipeline: &'b Self::Pipeline) {
        self.pipeline = Some(pipeline);
    }

    fn set_bind_group(&mut self, slot: u32, group: &'b Self::BindGroup) {
        self.bind_groups.push((slot, group, &[]));
    }

    fn finish(self) -> Self::Out {
        CtxOut::Render(self)
    }
}

impl<'a, 'b> DrawModel<'a, 'b> for RenderCtx<'a, 'b>
where
    'b: 'a,
{
    type BindGroup = wgpu::BindGroup;

    fn draw_mesh(
        &mut self,
        mesh: &'b model::Mesh,
        material: &'b model::Material,
        camera_bind_group: &'b Self::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group)
    }
    fn draw_model(&mut self, model: &'b model::Model, camera_bind_group: &'b Self::BindGroup) {
        self.draw_model_instanced(model, 0..1, camera_bind_group)
    }
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b model::Mesh,
        material: &'b model::Material,
        instances: Range<u32>,
        camera_bind_group: &'b Self::BindGroup,
    ) {
        self.set_vertex_buffer(0, &mesh.vertex_buffer);
        self.set_index_buffer(1, &mesh.index_buffer);
        self.set_bind_group(0, &material.bind_group);
        self.set_bind_group(1, &camera_bind_group);
        self.draw(0..mesh.num_elements, 0, instances);
    }
    fn draw_model_instanced(
        &mut self,
        model: &'b model::Model,
        instances: Range<u32>,
        camera_bind_group: &'b Self::BindGroup,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group);
        }
    }
}

impl<'a, 'b> IRenderContext<'a, 'b> for RenderCtx<'a, 'b>
where
    'b: 'a,
{
    type Buffer = wgpu::Buffer;
    fn draw(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.draw_cmd = Some(DrawCmd {
            indices,
            instances,
            base_vertex,
        });
    }
    fn set_index_buffer(&mut self, _slot: u32, buffer: &'b Self::Buffer) {
        self.index_buffer = Some(buffer)
    }
    fn set_vertex_buffer(&mut self, slot: u32, buffer: &'b Self::Buffer) {
        self.vertex_buffer = Some((slot, buffer))
    }
}

#[derive(Default)]
pub struct RenderCtx<'a, 'b>
where
    'b: 'a,
{
    pub(crate) bind_groups: Vec<(u32, &'b wgpu::BindGroup, &'b [DynamicOffset])>,
    pub(crate) vertex_buffer: Option<(u32, &'b wgpu::Buffer)>,
    pub(crate) index_buffer: Option<&'b wgpu::Buffer>,
    pub(crate) pipeline: Option<&'b wgpu::RenderPipeline>,
    pub(crate) draw_cmd: Option<DrawCmd>,
    _phantom: PhantomData<&'a ()>,
}

#[derive(Default)]
pub(super) struct DrawCmd {
    pub(super) indices: Range<u32>,
    pub(super) base_vertex: i32,
    pub(super) instances: Range<u32>,
}
