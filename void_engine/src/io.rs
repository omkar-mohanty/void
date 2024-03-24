use void_core::{Observer, Result};
use void_io::IoCmd;
use void_native::MpscSender;
use void_render::RenderEvent;
use void_ui::GuiEvent;

pub struct IoGuiObserver {
    pub cmd_sender: MpscSender<IoCmd>,
}

impl Observer<GuiEvent> for IoGuiObserver {
    fn update(&self, _event: &GuiEvent) -> Result<()> {
        Ok(())
    }
}

pub struct IoRenderObserver {
    pub cmd_sender: MpscSender<IoCmd>,
}

impl Observer<RenderEvent> for IoRenderObserver {
    fn update(&self, _event: &RenderEvent) -> Result<()> {
        Ok(())
    }
}
