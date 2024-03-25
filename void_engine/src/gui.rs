use void_core::{CmdSender, Observer, Result};
use void_io::IoEvent;
use void_native::MpscSender;
use void_render::RenderEvent;
use void_ui::GuiCmd;

pub struct GuiObserver {
    pub cmd_sender: MpscSender<GuiCmd>,
}

impl Observer<IoEvent> for GuiObserver {
    fn update(&self, event: &IoEvent) -> Result<()> {
        if let IoEvent::Input(input) = event {
            self.cmd_sender
                .send_blocking(GuiCmd::Input(input.clone()))?
        }
        Ok(())
    }
}

impl Observer<RenderEvent> for GuiObserver {
    fn update(&self, event: &RenderEvent) -> Result<()> {
        match event {
            RenderEvent::PassComplete => self.cmd_sender.send_blocking(GuiCmd::Pass)?,
        }
        Ok(())
    }
}
