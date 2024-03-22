use egui::{Context, FullOutput, RawInput};
use void_core::{Event, EventEmitter, EventListner};

#[cfg(not(target_arch = "wasm32"))]
mod native;

pub trait Gui: Fn(&Context) {}

pub enum GuiEvent {
    Input(RawInput),
    Output(FullOutput),
}

impl Event for GuiEvent {}

struct GuiEngine<S, R, U, D, T>
where
    S: EventEmitter<E = U>,
    R: EventListner<E = D>,
    U: Event,
    D: Event,
    T: Gui,
{
    context: Context,
    event_sender: S,
    event_receiver: R,
    state: T,
}

impl<S, R, U, D, T> GuiEngine<S, R, U, D, T>
where
    S: EventEmitter<E = U>,
    R: EventListner<E = D>,
    U: Event,
    D: Event,
    T: Gui,
{
    fn update(&mut self, raw_input: RawInput) -> FullOutput {
        self.context.run(raw_input, |_| {
            (self.state)(&self.context);
        })
    }
}
