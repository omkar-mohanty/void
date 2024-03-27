#[cfg(not(target_arch = "wasm32"))]
mod native;

mod gui;

use std::sync::Arc;
use void_core::{IEvent, IEventReceiver, ISubject};
use winit::window::Window;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum IoCmd {}

impl IEvent for IoCmd {}

#[derive(Clone, Hash, Copy, PartialEq, Eq)]
pub enum AppEvent {
    Output,
    Input,
    Resized,
    Redraw,
    Exit,
}

impl IEvent for AppEvent {}

pub struct IoEngine<S, R>
where
    S: ISubject<E = AppEvent>,
    R: IEventReceiver<IoCmd>,
{
    window: Arc<Window>,
    subject: S,
    receiver: R,
}

impl<S, R> IoEngine<S, R>
where
    S: ISubject<E = AppEvent>,
    R: IEventReceiver<IoCmd>,
{
    pub fn new(window: Arc<Window>, subject: S, receiver: R) -> Self {
        Self {
            subject,
            window,
            receiver,
        }
    }
}
