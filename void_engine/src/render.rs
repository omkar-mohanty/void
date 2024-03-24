use void_core::{CmdSender, Observer, Result};
use void_io::IoEvent;
use void_native::MpscSender;
use void_render::RenderCmd;
use void_ui::GuiEvent;

pub struct RendererObserver {
    pub cmd_sender: MpscSender<RenderCmd>,
}

impl Observer<IoEvent> for RendererObserver {
    fn update(&self, event: &IoEvent) -> Result<()> {
        use IoEvent::*;

        match event {
            Resized { height, width } => self.cmd_sender.send_blocking(RenderCmd::Resize {
                height: *height,
                width: *width,
            })?,
            _ => {}
        }

        Ok(())
    }
}

impl Observer<GuiEvent> for RendererObserver {
    fn update(&self, event: &GuiEvent) -> Result<()> {
        use GuiEvent::*;
        match event {
            Output(output) => {
                self.cmd_sender
                    .send_blocking(RenderCmd::GuiOutput(output.clone()))?;
            }
        }
        Ok(())
    }
}
