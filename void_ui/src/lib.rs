use egui::{Context, FullOutput, RawInput};
use void_core::{Command, Event};

#[cfg(not(target_arch = "wasm32"))]
mod native;

pub trait Gui: Fn(&Context) {}

pub enum GuiCmd {
    Input(RawInput),
    Pass,
}

impl Command for GuiCmd {}

struct GuiEngine<T>
where
    T: Gui,
{
    context: Context,
    state: T,
}

impl<T> GuiEngine<T>
where
    T: Gui,
{
    fn update(&mut self, raw_input: RawInput) -> FullOutput {
        self.context.run(raw_input, |_| {
            (self.state)(&self.context);
        })
    }
}
