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
}

impl IMeshFile for StlFile {
    fn get_indices(&self) -> Result<Vec<u32>> {
        let mesh = &self.mesh;
        let mut indices = Vec::with_capacity(mesh.vertices.len());
        let total_faces = mesh.faces.len();

        for i in 0..total_faces {
            indices.push((i * 3) as u32);
            indices.push((i * 3 + 1) as u32);
            indices.push((i * 3 + 2) as u32);
        }
        Ok(indices)
    }

    fn get_vertices(&self) -> Result<Vec<model::ModelVertex>> {
        let mesh = &self.mesh;

        let mut model_vertices = Vec::with_capacity(mesh.vertices.len());
        let mesh_vertices = &mesh.vertices;

        for face in mesh.faces.iter() {
            let indices = face.vertices;

            for i in 0..3 {
                let idx = indices[i];
                let vertex = model::ModelVertex {
                    position: mesh_vertices[idx].into(),
                    normal: face.normal.into(),
                    tex_coord: self.get_uv(&mesh_vertices[idx].into()),
                };

                model_vertices.push(vertex);
            }
        }

        Ok(model_vertices)
    }

    fn get_uv(&self, vertex: &[f32; 3]) -> [f32; 2] {
        let abs_x = vertex[0].abs();
        let abs_y = vertex[1].abs();
        let abs_z = vertex[2].abs();

        if abs_x >= abs_y && abs_x >= abs_z {
            // Project on the yz plane
            [(vertex[1] + 1.0) * 0.5, (vertex[2] + 1.0) * 0.5]
        } else if abs_y >= abs_x && abs_y >= abs_z {
            // Project on the xz plane
            [(vertex[0] + 1.0) * 0.5, (vertex[2] + 1.0) * 0.5]
        } else {
            // Project on the xy plane
            [(vertex[0] + 1.0) * 0.5, (vertex[1] + 1.0) * 0.5]
        }
    }
}
