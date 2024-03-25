use egui::Context;
use void_core::{ICmdReceiver, Result, ISubject, ISystem};

use crate::{Gui, GuiCmd, GuiEngine, GuiEvent};

impl<T, R, S> ISystem for GuiEngine<T, R, S>
where
    T: Gui + Send,
    R: ICmdReceiver<GuiCmd>,
    S: ISubject<E = GuiEvent> + Send,
{
    type C = GuiCmd;
    async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(cmd) = self.receiver.recv().await {
                self.handle_cmd(cmd);
            }
        }
    }
}

impl<T, R, S> GuiEngine<T, R, S>
where
    T: Gui,
    R: ICmdReceiver<GuiCmd>,
    S: ISubject<E = GuiEvent>,
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
