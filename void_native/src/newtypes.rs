use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use void_core::{Event, EventEmitter, EventListner, Result};

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

pub type NativeEventReceiver = MpscReceiver<NativeEvent>;
pub type NativeEventSender = MpscSender<NativeEvent>;

impl<T> EventEmitter for MpscSender<T>
where
    T: Event + 'static,
{
    type E = T;
    async fn send_event(&self, event: Self::E) -> Result<()> {
        self.0.send(event)?;
        Ok(())
    }
    fn send_event_blocking(&self, _event: Self::E) -> Result<()> {
        todo!()
    }
}

impl<T> EventListner for MpscReceiver<T>
where
    T: Event + 'static,
{
    type E = T;

    async fn receieve_event(&mut self) -> Result<Self::E> {
        let event = self.0.recv().await.unwrap();
        Ok(event)
    }

    fn receive_event_blocking(&mut self) -> Result<Self::E> {
        let event = self.0.blocking_recv().unwrap();
        Ok(event)
    }
}
