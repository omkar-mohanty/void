use std::fmt::Display;

use egui::{Context, FullOutput, RawInput};
use void_core::{CmdReceiver, Command, Event, Result, Subject};

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

pub trait Gui {
    fn show(&mut self, context: &Context);
}

pub enum GuiEvent {
    Output(FullOutput),
}

impl Event for GuiEvent {}

#[derive(Clone)]
pub enum GuiCmd {
    Input(RawInput),
    Pass,
}

impl Display for GuiCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GuiCmd::*;
        match self {
            Input(_) => f.write_str("Input"),
            Pass => f.write_str("Pass"),
        }
    }
}

impl Command for GuiCmd {}

pub struct GuiEngine<T, R, S>
where
    T: Gui,
    R: CmdReceiver<GuiCmd>,
    S: Subject<E = GuiEvent>,
{
    context: Context,
    receiver: R,
    gui: T,
    subject: S,
}

impl<T, R, S> GuiEngine<T, R, S>
where
    T: Gui,
    R: CmdReceiver<GuiCmd>,
    S: Subject<E = GuiEvent>,
{
    fn handle_cmd(&mut self, cmd: GuiCmd) {
        use GuiCmd::*;
        match cmd {
            Input(raw_input) => {
                let output = self.update(raw_input);
                self.subject.notify(GuiEvent::Output(output));
            }
            Pass => {}
        }
    }
    fn update(&mut self, raw_input: RawInput) -> FullOutput {
        self.context.run(raw_input, |_| {
            self.gui.show(&self.context);
        })
    }
}
