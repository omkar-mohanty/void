use std::collections::BTreeMap;

use crate::{
    api::{BufferType, CommandListIndex, GpuApi},
    model::Vertex,
};

use super::GpuResource;
use wgpu::{
    rwh::{HasDisplayHandle, HasWindowHandle},
    util::DeviceExt,
    SurfaceTarget,
};
