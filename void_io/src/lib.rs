use std::sync::Arc;

use void_gpu::api::{Displayable, Gpu};

pub struct IoEngine<'a, T: Displayable<'a>> {
    gpu: Arc<Gpu<'a, T>>,
}
