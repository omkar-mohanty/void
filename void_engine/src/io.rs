use void_core::{IObserver, Result};
use void_io::IoCmd;
use void_native::MpscSender;
use void_render::RenderEvent;

pub struct IoRenderObserver {
    pub cmd_sender: MpscSender<IoCmd>,
}

impl IObserver<RenderEvent> for IoRenderObserver {
    fn update(&self, _event: &RenderEvent) -> Result<()> {
        Ok(())
    }
}
