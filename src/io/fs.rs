use crate::{gpu::Gpu, model, texture};
use anyhow::Result;
use std::{ffi::OsStr, fs::OpenOptions, path::PathBuf};

pub trait IMeshFile {
    fn get_meshes(&self, gpu: &Gpu) -> Vec<model::Mesh>;
    fn get_materials(&self, gpu: &Gpu) -> Vec<model::Material>;
    fn get_model(&self, gpu: &Gpu) -> model::Model {
        let meshes = self.get_meshes(gpu);
        let materials = self.get_materials(gpu);

        model::Model { meshes, materials }
    }
}

pub struct MeshFile {
    inner: Box<dyn IMeshFile>,
}

impl MeshFile {
    pub fn new(path: PathBuf) -> Result<Self> {
        valid_file(&path)?;

        let ext = OsStr::to_str(path.extension().unwrap()).unwrap();

        let inner = match ext {
            "stl" => Box::new(StlFile { path }),
            _ => {
                anyhow::bail!("Unsupported file format")
            }
        };

        Ok(Self { inner })
    }
}

struct StlFile {
    path: PathBuf,
}

impl IMeshFile for StlFile {
    fn get_meshes(&self, gpu: &Gpu) -> Vec<model::Mesh> {
        let mut file = OpenOptions::new().read(true).open(self.path).unwrap();

        let stl = stl_io::read_stl(&mut file).unwrap();
    }

    fn get_materials(&self, _gpu: &Gpu) -> Vec<model::Material> {
        Vec::new()
    }
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

#[cfg(test)]
mod test {
    #[test]
    fn test_mesh_file() {}
}
