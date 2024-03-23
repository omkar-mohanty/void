use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use void_core::{CmdReceiver, CmdSender, Command, Event, Result};

pub enum NativeEvent {
    Render,
}

impl Event for NativeEvent {}

pub struct MpscReceiver<T>(pub UnboundedReceiver<T>);
pub struct MpscSender<T>(pub UnboundedSender<T>);

pub fn create_mpsc_channel<T>() -> (MpscSender<T>, MpscReceiver<T>) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<T>();
    (MpscSender(sender), MpscReceiver(receiver))
}

impl<T: Command + 'static> CmdSender<T> for MpscSender<T> {
    async fn send(&self, cmd: T) -> Result<()> {
        self.0.send(cmd)?;
        Ok(())
    }
}

impl<T: Command + 'static> CmdReceiver<T> for MpscReceiver<T> {
    async fn recv(&mut self) -> Option<T> {
        self.0.recv().await
    }
}
