use crate::io::fs::IMeshFile;
use crate::model;

use anyhow::Result;
use std::{fs::OpenOptions, path::PathBuf};
use stl_io::IndexedMesh;

pub struct StlFile {
    mesh: IndexedMesh,
}

impl StlFile {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let mut file = OpenOptions::new().read(true).open(&path)?;

        let mesh = stl_io::read_stl(&mut file)?;
        Ok(Self { mesh })
    }

    pub fn generate_uv(vertex: &[f32; 3]) -> [f32; 2] {
        [vertex[0], vertex[1]]
    }
}

impl IMeshFile for StlFile {
    fn get_indices(&self) -> Result<Vec<u32>> {
        let mesh = &self.mesh;
        let vertices = &mesh.vertices;
        let total_positions = mesh.vertices.len() * 3;
        let mut indices = Vec::with_capacity(total_positions);

        for i in 0..vertices.len() {
            indices.push((i * 3) as u32);
            indices.push((i * 3 + 1) as u32);
            indices.push((i * 3 + 2) as u32);
        }
        Ok(indices)
    }
    fn get_vertices(&self) -> Result<Vec<model::ModelVertex>> {
        let mesh = &self.mesh;

        let total_positions = mesh.vertices.len() * 3;
        let vertices = &mesh.vertices;
        let faces = &mesh.faces;

        let mut positions = Vec::with_capacity(total_positions);
        let mut normals = Vec::with_capacity(total_positions);

        for i in 0..vertices.len() {
            positions.push(vertices[i][0]);
            positions.push(vertices[i][1]);
            positions.push(vertices[i][2]);
        }

        for i in 0..vertices.len() {
            normals.push(faces[i].normal[0]);
            normals.push(faces[i].normal[1]);
            normals.push(faces[i].normal[2]);
        }

        let vertices = (0..positions.len() / 3)
            .map(|i| model::ModelVertex {
                position: [positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]],
                normal: [normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]],
            })
            .collect::<Vec<_>>();

        Ok(vertices)
    }

    fn get_tex_coordinates(&self) -> Result<Vec<[f32; 2]>> {
        let mesh = &self.mesh;
        let mut tex_coords = Vec::with_capacity(mesh.vertices.len());
        for i in 0..mesh.vertices.len() {
            let position = [
                mesh.vertices[i][0],
                mesh.vertices[i][1],
                mesh.vertices[i][2],
            ];
            let tex_coord = Self::generate_uv(&position);
            tex_coords.push(tex_coord)
        }

        Ok(tex_coords)
    }
}
