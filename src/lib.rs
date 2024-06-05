extern crate nalgebra as na;

pub mod app;
mod camera;
mod db;
pub mod gpu;
mod gui;
mod hdr;
mod io;
mod light;
mod model;
mod resource;
mod texture;

use crate::db::Id;
use crate::model::{InstanceRaw, ModelVertex, Vertex};

use camera::{CameraController, CameraUniform, Projection, StaticCamera};
use db::DB;
use gpu::Gpu;
use io::Controller;
use light::LightUniform;
use model::DrawLight;
use model::DrawModel;
use std::sync::{Arc, RwLock};
use texture::Texture;
use wgpu::util::DeviceExt;
use winit::{event::*, window::Window};

fn create_render_pipeline(
    device: &Gpu,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    topology: wgpu::PrimitiveTopology, // NEW!
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let device = &device.device;
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{:?}", shader)),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology, // NEW!
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual, // UDPATED!
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    })
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

impl Controller for Arc<RwLock<CameraController>> {
    fn process_events(&self, ctx: &KeyEvent) {
        let mut camera_write = self.write().unwrap();
        camera_write.process_key(ctx);
    }
}

type ModelDB = DB<ModelEntry>;
type BindGroupDB = DB<BindGroupEntry>;
type PipelineDB = DB<PipelineEntry>;

enum PipelineEntry {
    Render(wgpu::RenderPipeline),
    Compute(wgpu::ComputePipeline),
}

struct ModelEntry {
    model: model::Model,
    instances: Vec<model::Instance>,
    instance_buffer: wgpu::Buffer,
}

struct BindGroupEntry {
    bind_group: Option<wgpu::BindGroup>,
    layout: wgpu::BindGroupLayout,
}

struct Resources {
    pub pipeline_db: RwLock<PipelineDB>,
    pub bind_group_db: RwLock<BindGroupDB>,
    pub model_db: RwLock<ModelDB>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            pipeline_db: RwLock::default(),
            bind_group_db: RwLock::default(),
            model_db: RwLock::default(),
        }
    }
}

struct Renderer {
    gpu: Arc<Gpu>,
    window: Arc<Window>,
    camera_controller: Arc<RwLock<CameraController>>,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    camera: Arc<RwLock<StaticCamera>>,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: Id,
    depth_texture: Option<texture::Texture>,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    hdr: hdr::HdrPipeline,
    bind_group_db: BindGroupDB,
}

impl Renderer {
    async fn new(
        window: Arc<Window>,
        gpu: Arc<Gpu>,
        camera_controller: Arc<RwLock<CameraController>>,
        static_camera: Arc<RwLock<StaticCamera>>,
    ) -> Self {
        let device = &gpu.device;

        let size = window.inner_size();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let mut camera_uniform = CameraUniform::new();
        let camera = static_camera.read().unwrap();
        let projection = Projection::with_aspect(size.width as f32, size.height as f32);
        camera_uniform.update_view_projection(&projection, &*camera);
        drop(camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let light_uniform = light::LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _paddding: 0,
        };

        let hdr = hdr::HdrPipeline::new(&gpu);

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light V8"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                }],
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        let depth_texture = None;

        // lib.rs
        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            create_render_pipeline(
                &gpu,
                &layout,
                hdr.format(),
                Some(texture::Texture::DEPTH_FORMAT),
                &[ModelVertex::desc()],
                wgpu::PrimitiveTopology::TriangleList,
                shader,
            )
        };

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            create_render_pipeline(
                &gpu,
                &render_pipeline_layout,
                hdr.format(),
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                wgpu::PrimitiveTopology::TriangleList,
                shader,
            )
        };

        let mut bind_group_db = BindGroupDB::default();

        let camera_bind_group = bind_group_db.insert(BindGroupEntry {
            bind_group: Some(camera_bind_group),
            layout: camera_bind_group_layout,
        });

        Self {
            gpu,
            depth_texture,
            hdr,
            size,
            render_pipeline,
            window,
            camera: static_camera,
            camera_uniform,
            camera_bind_group,
            camera_buffer,
            light_buffer,
            light_uniform,
            light_bind_group,
            light_render_pipeline,
            camera_controller,
            bind_group_db,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;

            let device = &self.gpu.device;

            let mut config_write = self.gpu.get_config_mut();
            let surface = &self.gpu.surface;

            config_write.width = new_size.width;
            config_write.height = new_size.height;

            surface.configure(device, &config_write);
            self.depth_texture = Some(texture::Texture::create_depth_texture(
                &device,
                &config_write,
                "depth_texture",
            ));
            self.hdr
                .resize(&self.gpu, self.size.width, self.size.height);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        self.window().request_redraw();
        false
    }

    fn update(&mut self) {
        let mut camera = self.camera.write().unwrap();
        self.camera_controller
            .write()
            .unwrap()
            .update_camera(&mut *camera);

        let (width, height) = self
            .gpu
            .get_config_read(|config| (config.width as f32, config.height as f32));

        let projection = Projection::with_aspect(width, height);

        self.camera_uniform
            .update_view_projection(&projection, &mut *camera);

        // Update the light
        let old_position: nalgebra::Vector3<_> = self.light_uniform.position.into();
        let isom = na::Isometry3::new(old_position, *na::Vector3::y_axis());

        let queue = &self.gpu.queue;

        self.light_uniform.position = isom.translation.into();
        queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render_models<'a>(
        &mut self,
        models: impl Iterator<Item = &'a ModelEntry>,
    ) -> Result<(), wgpu::SurfaceError> {
        let view = self.gpu.get_current_view();
        let device = &self.gpu.device;

        let camera_bind_group_entry = self.bind_group_db.get(self.camera_bind_group);
        let camera_bind_group = camera_bind_group_entry.bind_group.as_ref().unwrap();

        let config = self.gpu.get_config();

        if let None = self.depth_texture {
            self.depth_texture = Some(Texture::create_depth_texture(
                device,
                &config,
                "Depth Texture",
            ));
        }

        let depth_tex = self.depth_texture.as_ref().unwrap();

        let mut encoder = self.gpu.create_cmd_encoder();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.hdr.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_tex.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            for entry in models {
                let model = &entry.model;
                let instances = &entry.instances;
                let instane_buffer = &entry.instance_buffer;

                //render_pass.set_pipeline(&self.light_render_pipeline);
                //render_pass.draw_light_model(model, camera_bind_group, &self.light_bind_group);

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_vertex_buffer(1, instane_buffer.slice(..));

                render_pass.draw_model_instanced(
                    model,
                    0..instances.len() as u32,
                    camera_bind_group,
                    &self.light_bind_group,
                )
            }
        }

        self.hdr.process(&mut encoder, &view);

        self.gpu.submit_cmd(encoder.finish());
        Ok(())
    }
}
