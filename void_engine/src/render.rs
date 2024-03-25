use void_core::{ICmdSender, IObserver, Result};
use void_io::IoEvent;
use void_native::MpscSender;
use void_render::RenderCmd;

pub struct RendererObserver {
    pub cmd_sender: MpscSender<RenderCmd>,
}

impl IObserver<IoEvent> for RendererObserver {
    fn update(&self, event: &IoEvent) -> Result<()> {
        use IoEvent::*;

        match event {
            Redraw => self.cmd_sender.send_blocking(RenderCmd::Render)?,
            _ => {}
        }

        Ok(())
    }
}
