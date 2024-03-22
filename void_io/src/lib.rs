use egui::Context;
use void_core::{Event, EventEmitter, EventListner};
use winit::window::Window;

pub enum IoEvent {
    Output(Vec<u8>),
}

impl Event for IoEvent {}

pub struct IoEngine<S, R, U, D>
where
    S: EventEmitter<E = U>,
    R: EventListner<E = D>,
    U: Event,
    D: Event,
{
    context: Context,
    state: egui_winit::State,
    event_receiver: R,
    event_sender: S,
}

impl<S, R, U, D> IoEngine<S, R, U, D>
where
    S: EventEmitter<E = U>,
    R: EventListner<E = D>,
    U: Event,
    D: Event,
{
    pub fn new(context: Context, window: Window, event_sender: S, event_receiver: R) -> Self {
        let viewport_id = context.viewport_id();
        let state = egui_winit::State::new(context.clone(), viewport_id, &window, None, None);
        Self {
            context,
            state,
            event_receiver,
            event_sender,
        }
    }
}
