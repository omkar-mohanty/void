use crate::{
    api::{Displayable, Gpu, Texture, TextureError},
    model,
};
use std::{
    fs,
    io::{BufReader, Cursor},
    path::PathBuf,
};
use thiserror::Error;
use tobj::LoadError;
use wgpu::util::DeviceExt;

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("learn-wgpu") {
        origin = format!("{}/learn-wgpu", origin);
    }
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

pub fn load_texture<'a, T: Displayable<'a>>(
    file_name: &PathBuf,
    gpu: &Gpu<'a, T>,
) -> Result<Texture, IoError> {
    let device = &gpu.device;
    let queue = &gpu.queue;
    let data = std::fs::read(file_name)?;
    Ok(Texture::from_bytes(
        device,
        queue,
        &data,
        &file_name.display().to_string(),
    )?)
}

pub fn load_model<'a, T: Displayable<'a>>(
    file_path: &PathBuf,
    gpu: &Gpu<'a, T>,
) -> Result<model::Model, IoError> {
    let device = &gpu.device;
    let obj_text = fs::read_to_string(file_path)?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let parent = file_path.parent().unwrap();

    let (models, obj_materials) = tobj::load_obj_buf(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |mat_path| {
            let full_path = if let Some(parent) = file_path.parent() {
                parent.join(mat_path)
            } else {
                mat_path.to_owned()
            };
            tobj::load_mtl(full_path)
        },
    )?;

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    });

    let mut materials = Vec::new();
    let obj_materials = obj_materials?;
    for m in obj_materials {
        let diffuse_texture = load_texture(&parent.join(&m.diffuse_texture), gpu)?;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: None,
        });

        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
            bind_group,
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_path)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_path)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::Mesh {
                name: file_path.display().to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}

#[derive(Error, Debug)]
pub enum IoError {
    #[error("Io Error  {0}")]
    Error(#[from] std::io::Error),
    #[error("Texture Error {0}")]
    TextureError(#[from] TextureError),
    #[error("Load Error {0}")]
    LoadError(#[from] LoadError),
}
