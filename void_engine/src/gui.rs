use void_core::{CmdSender, Observer};
use void_io::IoEvent;
use void_native::MpscSender;
use void_render::RenderEvent;
use void_ui::GuiCmd;

pub struct GuiRenderObserver {
    cmd_sender: MpscSender<GuiCmd>,
}

impl Observer<RenderEvent> for GuiRenderObserver {
    fn update(&self, event: &RenderEvent) {
        match event {
            RenderEvent::PassComplete => {
                let _ = self.cmd_sender.send(GuiCmd::Pass);
            }
        }
    }
}

pub struct GuiIoObserver {
    cmd_sender: MpscSender<GuiCmd>,
}

impl Observer<IoEvent> for GuiIoObserver {
    fn update(&self, event: &IoEvent) {
        if let IoEvent::Input(input) = event {
            let _ = self.cmd_sender.send(GuiCmd::Input(input.clone()));
        }
    }
}
