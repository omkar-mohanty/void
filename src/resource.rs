use std::{path::PathBuf, str::FromStr};

use crate::{gpu::Gpu, model, io::fs::{MeshFile,IMeshFile}};

pub async fn load_model(
    file_name: &str,
    gpu: &Gpu,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let path = PathBuf::from_str(file_name)?;

    let mesh_file = MeshFile::new(path)?;

    let positions = mesh_file.get_vertices()?;
    let indices = mesh_file.get_indices()?;
    let tex_coords = mesh_file.get_tex_coordinates()?;

    Ok(model::Model { meshes, materials })
}
