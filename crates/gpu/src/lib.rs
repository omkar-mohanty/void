use std::collections::BTreeMap;
use std::{env, mem};
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, RwLock};
use wgpu::util::DeviceExt;

pub struct ResourceMgr<T> {
    res_map: BTreeMap<Id, T>,
    next_id: AtomicUsize
}

impl<T> ResourceMgr<T> {
    pub fn new() -> Self {
        Self {
            res_map: BTreeMap::new(),
            next_id: AtomicUsize::new(0),
        }
    }

    pub fn remove_buffer(&mut self, id: Id) {
        self.res_map.remove(&id);
    }

    pub fn insert_buffer(&mut self, buffer: T) -> Id {
        let key = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.res_map.insert(key, buffer);
        key
    }

    pub fn get(&self, id: Id) -> &T {
        self.res_map.get(&id).unwrap()
    }

    pub fn get_mut(&mut self, id: Id) -> &mut T {
        self.res_map.get_mut(&id).unwrap()
    }
}


pub struct IndexBuffer {
    buffer: Id,
    count: usize,
    gpu: Gpu,
}

impl IndexBuffer {
    pub fn new(gpu: &Gpu) -> Self {
        let gpu = gpu.clone();
        let buffer = gpu.create_buffer(wgpu::BufferUsages::INDEX, &[]);

        Self {
            buffer,
            gpu,
            count: 0,
        }
    }

    pub fn new_with_data(gpu: &Gpu, data: &[u8]) -> Self {
        assert!(
            data.len() % 3 == 0,
            "The number of indices must be a multiple of 3!"
        );
        let gpu = gpu.clone();
        let buffer = gpu.create_buffer(wgpu::BufferUsages::INDEX, data);

        Self {
            buffer,
            gpu,
            count: data.len(),
        }
    }

    pub fn fill(&self, offset: u64, data: &[u8]) {
        assert!(
            data.len() % 3 == 0,
            "The number of indices must be a multiple of 3!"
        );
        self.gpu.write_buffer(self.buffer, |buffer| {
            self.gpu.queue.write_buffer(buffer, offset, data);
        });
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn triangles(&self) -> usize {
        self.count / 3
    }
}

pub struct VertexBuffer {
    buffer: Id,
    gpu: Gpu,
}

impl VertexBuffer {
    pub fn new(gpu: &Gpu) -> Self {
        let buffer = gpu.create_buffer(wgpu::BufferUsages::VERTEX, &[]);
        let gpu = gpu.clone();

        Self { buffer, gpu }
    }

    pub fn new_with_data(gpu: &Gpu, data: &[u8]) -> Self {
        let gpu = gpu.clone();
        let buffer = gpu.create_buffer(wgpu::BufferUsages::INDEX, data);

        Self { buffer, gpu }
    }

    pub fn fill(&self, offset: u64, data: &[u8]) {
        self.gpu.write_buffer(self.buffer, |buffer| {
            self.gpu.queue.write_buffer(buffer, offset, data);
        });
    }
}

pub struct InstanceBuffer {
    buffer: Id,
    gpu: Gpu,
}

impl InstanceBuffer {
    pub fn new(gpu: &Gpu) -> Self {
        let buffer = gpu.create_buffer(wgpu::BufferUsages::VERTEX, &[]);
        let gpu = gpu.clone();

        Self { buffer, gpu }
    }

    pub fn new_with_data(gpu: &Gpu, data: &[u8]) -> Self {
        let gpu = gpu.clone();
        let buffer = gpu.create_buffer(wgpu::BufferUsages::INDEX, data);

        Self { buffer, gpu }
    }

    pub fn fill(&self, offset: u64, data: &[u8]) {
        self.gpu.write_buffer(self.buffer, |buffer| {
            self.gpu.queue.write_buffer(buffer, offset, data);
        });
    }
}

pub struct UniformBuffer {
    buffer: Id,
    gpu: Gpu,
}

impl UniformBuffer {
    pub fn new(gpu: &Gpu) -> Self {
        let gpu = gpu.clone();
        let buffer = gpu.create_buffer(wgpu::BufferUsages::UNIFORM, &[]);
        Self { buffer, gpu }
    }

    pub fn new_with_data(gpu: &Gpu, data: &[u8]) -> Self {
        let gpu = gpu.clone();
        let buffer = gpu.create_buffer(wgpu::BufferUsages::INDEX, data);

        Self { buffer, gpu }
    }

    pub fn fill(&self, offset: u64, data: &[u8]) {
        self.gpu.write_buffer(self.buffer, |buffer| {
            self.gpu.queue.write_buffer(buffer, offset, data);
        });
    }
}

type Id = usize;

#[derive(Clone)]
pub struct Gpu {
    device: Arc<wgpu::Device>,
    adapter: Arc<wgpu::Adapter>,
    queue: Arc<wgpu::Queue>,
    buffer_mgr: Arc<RwLock<ResourceMgr<wgpu::Buffer>>>,
    cmds: Arc<RwLock<ResourceMgr<wgpu::CommandEncoder>>>
}

impl Gpu {
    pub fn create_buffer(&self, usage: wgpu::BufferUsages, data: &[u8]) -> Id {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("gpu_buffer_init_desc"),
                contents: data,
                usage,
            });
        let mut mgr_write = self.buffer_mgr.write().unwrap();
        mgr_write.insert_buffer(buffer)
    }

    pub fn write_buffer<F: FnOnce(&wgpu::Buffer)>(&self, id: Id, func: F) {
        let map_read = self.buffer_mgr.read().unwrap();
        let buffer = map_read.get(id);
        func(buffer);
    }

    pub fn create_cmd(&self) -> Id {
        let mut cmds_write = self.cmds.write().unwrap();
        let cmd_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label:Some("gpu_create_cmd")
        });
        cmds_write.insert_buffer(cmd_encoder)
    }

    pub fn record_cmd<F: FnMut(&mut wgpu::CommandEncoder)>(&self, id: Id,mut func: F) {
        let mut cmds_write = self.cmds.write().unwrap();
        let cmd  = cmds_write.get_mut(id);
        func(cmd);
    }
}

pub struct RenderPass<'a> {
    rpass: wgpu::RenderPass<'a>,
}

pub struct MeshVertex(pub [f32; 3]);

static VERTEX_BUFFER_LAYOUT_DESC: wgpu::VertexBufferLayout = wgpu::VertexBufferLayout {
    array_stride: mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[wgpu::VertexAttribute {
        offset: 0,
        format: wgpu::VertexFormat::Float32x3,
        shader_location: 0,
    }],
};

pub trait IBuffer {}

impl IBuffer for IndexBuffer {}

impl Gpu {
    fn create_vertex_buffer(&self) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("gpu_vertex_buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::VERTEX,
            })
    }
}
