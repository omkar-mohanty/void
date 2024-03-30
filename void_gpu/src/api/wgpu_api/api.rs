use std::{collections::BTreeMap, sync::Arc};

use crate::{CommandListIndex, Displayable, IGpu, GpuResource};

pub struct Gpu<'a, T>
where
    T: Displayable<'a>,
{
    gpu_resource: Arc<GpuResource<'a, T>>,
    node_id: &'a [u8; 6],
    commands: BTreeMap<CommandListIndex, wgpu::CommandEncoder>,
}
