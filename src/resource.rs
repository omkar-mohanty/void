use crate::{
    gpu::Gpu,
    io::fs::{IMeshFile, MeshFile},
    model, texture,
};
use std::path::PathBuf;
use wgpu::util::DeviceExt;

pub async fn load_model(
    path: PathBuf,
    gpu: &Gpu,
) -> anyhow::Result<model::Model> {
    let (device, queue) = (&gpu.device, &gpu.queue);
    let file_name = path.display().to_string();
    let mesh_file = MeshFile::new(path)?;

    let vertices = mesh_file.get_vertices()?;
    let indices = mesh_file.get_indices()?;

    let default_texture = texture::Texture::random_texture(device, queue)?;

    let bind_group = texture::Texture::load(gpu, &default_texture);

    let materials = vec![model::Material {
        bind_group,
        diffuse_texture: default_texture,
        name: "Default texture".to_string(),
    }];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{:?} Vertex Buffer", file_name)),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{:?} Index Buffer", file_name)),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let meshes = vec![model::Mesh {
        name: file_name.to_string(),
        vertex_buffer,
        index_buffer,
        material: 0,
        num_elements: indices.len() as u32,
    }];

    Ok(model::Model { meshes, materials })
}
