use crate::model;
use anyhow::Result;
use std::{ffi::OsStr, path::PathBuf};

mod obj;
mod stl;

use obj::*;
use stl::*;

pub trait IMeshFile {
    fn get_vertices(&self) -> Result<Vec<model::ModelVertex>>;
    fn get_indices(&self) -> Result<Vec<u32>>;
    fn get_uv(&self, vertex: &[f32; 3]) -> [f32; 2];
}

pub struct MeshFile {
    inner: Box<dyn IMeshFile>,
}

fn valid_file(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("File does not exist")
    }
    if path.extension().is_none() {
        anyhow::bail!("No extension")
    }
    Ok(())
}

impl MeshFile {
    pub fn new(path: PathBuf) -> Result<Self> {
        valid_file(&path)?;

        let ext = OsStr::to_str(path.extension().unwrap()).unwrap();

        let inner = match ext {
            "stl" => Box::new(StlFile::new(&path)?),
            _ => {
                anyhow::bail!("Unsupported file format")
            }
        };

        Ok(Self { inner })
    }
}

impl IMeshFile for MeshFile {
    fn get_vertices(&self) -> Result<Vec<model::ModelVertex>> {
        self.inner.get_vertices()
    }
    fn get_uv(&self, vertex: &[f32; 3]) -> [f32; 2] {
        self.inner.get_uv(vertex)
    }
    fn get_indices(&self) -> Result<Vec<u32>> {
        self.inner.get_indices()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::str::FromStr;

    static MODEL_PATH: &'static str = "./models";

    #[test]
    fn test_mesh_file() -> Result<()> {
        let stl_path = PathBuf::from_str(&MODEL_PATH).unwrap().join("test.stl");
        let test_mesh_file = MeshFile::new(stl_path)?;
        let meshes = test_mesh_file.get_vertices()?;
        assert!(meshes.len() != 0);
        Ok(())
    }
}
