use egui::Context;
use void_core::{CmdReceiver, Result, Subject, System};

use crate::{Gui, GuiCmd, GuiEngine, GuiEvent};

impl<T, R, S> System for GuiEngine<T, R, S>
where
    T: Gui + Send,
    R: CmdReceiver<GuiCmd>,
    S: Subject<E = GuiEvent> + Send,
{
    type C = GuiCmd;
    async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(cmd) = self.receiver.recv().await {
                self.handle_cmd(cmd)?;
            }
        }
    }
}

impl<T, R, S> GuiEngine<T, R, S>
where
    T: Gui,
    R: CmdReceiver<GuiCmd>,
    S: Subject<E = GuiEvent>,
{
    pub fn new(context: Context, receiver: R, subject: S, gui: T) -> Self {
        Self {
            context,
            receiver,
            gui,
            subject,
        }
    }
}

pub struct NativeGui {}

impl Gui for NativeGui {
    fn show(&mut self, context: &Context) {
        use egui::Align2;
        egui::Window::new("Streamline CFD")
            // .vscroll(true)
            .default_open(true)
            .max_width(1000.0)
            .max_height(800.0)
            .default_width(800.0)
            .resizable(true)
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(&context, |ui| {
                if ui.add(egui::Button::new("Click me")).clicked() {
                    println!("PRESSED")
                }

                ui.label("Slider");
                // ui.add(egui::Slider::new(_, 0..=120).text("age"));
                ui.end_row();

                // proto_scene.egui(ui);
            });
    }
}
