use std::{iter, sync::Arc};

use crate::{
    model::{Vertex, INDICES, VERTICES},
    pipeline, IRenderer, RendererBuilder,
};
use void_core::{IBuilder, IEventReceiver, ISubject, ISystem, Result};
use wgpu::util::DeviceExt;

use super::{RenderCmd, RenderEvent, WindowResource};

pub struct ModelRendererBuilder<'a, P, R>
where
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    resource: Option<Arc<WindowResource<'a>>>,
    subject: Option<P>,
    receiver: Option<R>,
}

impl<P, R> Default for ModelRendererBuilder<'_, P, R>
where
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    fn default() -> Self {
        Self {
            resource: None,
            subject: None,
            receiver: None,
        }
    }
}

impl<'a, P, R> IBuilder for ModelRendererBuilder<'a, P, R>
where
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    type Output = ModelRenderer<'a, P, R>;

    async fn build(self) -> Result<Self::Output> {
        let ModelRendererBuilder {
            resource,
            subject,
            receiver,
        } = self;

        if resource.is_none() || subject.is_none() || receiver.is_none() {
            todo!("Fail Error")
        }

        let resource = resource.unwrap();
        let subject = subject.unwrap();
        let receiver = receiver.unwrap();

        let vertex_buffer = resource
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model Renderer Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = resource
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model Renderer INdex Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        let device = &resource.device;
        let color_format = resource.config.format;
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Model Renderer Pipeline"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Model Renderer Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        };

        let pipeline = pipeline::create_render_pipeline(
            &device,
            &layout,
            color_format,
            None,
            &[Vertex::desc()],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
        );

        Ok(ModelRenderer {
            receiver,
            resource,
            index_buffer,
            vertex_buffer,
            subject,
            pipeline,
        })
    }
}

impl<'a, P, R> IBuilder for RendererBuilder<ModelRendererBuilder<'a, P, R>, ModelRenderer<'a, P, R>>
where
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    type Output = ModelRenderer<'a, P, R>;

    async fn build(self) -> Result<Self::Output> {
        let res = self.builder.build().await?;
        Ok(res)
    }
}

impl<'a, P, R> RendererBuilder<ModelRendererBuilder<'a, P, R>, ModelRenderer<'a, P, R>>
where
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    fn new_model() -> Self {
        Self {
            builder: ModelRendererBuilder::default(),
        }
    }

    pub fn set_resource(mut self, resource: Arc<WindowResource<'a>>) -> Self {
        self.builder.resource = Some(resource);
        self
    }

    pub fn set_receiver(mut self, receiver: R) -> Self {
        self.builder.receiver = Some(receiver);
        self
    }

    pub fn set_subject(mut self, subject: P) -> Self {
        self.builder.subject = Some(subject);
        self
    }
}

impl<'a, P, R> ISystem for ModelRenderer<'a, P, R>
where
    P: ISubject<E = RenderEvent> + Send,
    R: IEventReceiver<RenderCmd>,
{
    type C = RenderCmd;

    async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(cmd) = self.receiver.recv().await {
                log::info!("Render Engine Received : {cmd}");
                self.handle_cmd(cmd)?;
            }
        }
    }

    fn run_blocking(&mut self) -> Result<()> {
        if let Some(cmd) = self.receiver.recv_blockding() {
            log::info!("Render Engine Received : {cmd}");
            self.handle_cmd(cmd)?;
        }
        Ok(())
    }
}

pub struct ModelRenderer<'a, P, R>
where
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    resource: Arc<WindowResource<'a>>,
    subject: P,
    receiver: R,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl<'a, P, R> ModelRenderer<'a, P, R>
where
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    pub fn builder() -> RendererBuilder<ModelRendererBuilder<'a, P, R>, Self> {
        RendererBuilder::new_model()
    }
    fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        let surface = &self.resource.surface;
        let device = &self.resource.device;
        let pipeline = &self.pipeline;
        let queue = &self.resource.queue;

        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw(0..3, 0..1);
        }

        queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn handle_cmd(
        &mut self,
        render_event: RenderCmd,
    ) -> std::result::Result<(), wgpu::SurfaceError> {
        use RenderCmd::*;
        match render_event {
            Render => self.render()?,
        };
        self.subject.notify(RenderEvent::PassComplete);
        log::info!("Render Notified");
        Ok(())
    }
}

impl<'a, P, R> IRenderer for ModelRenderer<'a, P, R>
where
    P: ISubject<E = RenderEvent>,
    R: IEventReceiver<RenderCmd>,
{
    async fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        self.render()
    }
    fn render_blocking(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        self.render()
    }
}
