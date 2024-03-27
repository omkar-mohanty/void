use crate::{FutError, IEvent, IEventReceiver, IEventSender};
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use egui::Event;
use std::{
    future::Future,
    sync::{Arc, Condvar, Mutex},
    task::{Poll, Waker},
};

pub fn create_unbounded_mpm<T>() -> (MpmcSender<T>, MpmcReceiver<T>) {
    let (send, recv) = unbounded();
    (MpmcSender(send), MpmcReceiver { state: Some(recv) })
}

pub struct MpmcReceiver<T> {
    state: Option<Receiver<T>>,
}
#[derive(Clone)]
pub struct MpmcSender<T>(pub Sender<T>);

impl<T: IEvent + 'static + Send> IEventSender<T> for MpmcSender<T> {
    async fn send(&self, cmd: T) -> Result<(), FutError<T>> {
        self.0.send(cmd)?;
        Ok(())
    }

    fn send_blocking(&self, cmd: T) -> Result<(), FutError<T>> {
        self.0.send(cmd)?;
        Ok(())
    }
}

struct InnerState<T> {
    pub recv: Receiver<T>,
    pub result: Option<std::result::Result<T, FutError<T>>>,
    pub waker: Option<Waker>,
    pub completed: bool,
}

struct RecvFuture<T> {
    state: Arc<(Mutex<InnerState<T>>, Condvar)>,
}

impl<T: Send + 'static> RecvFuture<T> {
    pub fn new(recv: Receiver<T>) -> Self {
        let state = InnerState {
            recv,
            result: None,
            waker: None,
            completed: false,
        };

        let state = Arc::new((Mutex::new(state), Condvar::new()));

        let thread_state = Arc::clone(&state);

        std::thread::spawn(move || {
            let (state, _cvar) = &*thread_state;

            loop {
                let mut state = state.lock().unwrap();

                let res = state.recv.try_recv();

                match res {
                    Ok(event) => {
                        state.result = Some(Ok(event));
                        state.completed = true;
                    }
                    Err(msg) => {
                        if let TryRecvError::Disconnected = msg {
                            state.result = Some(Err(FutError::from(msg)));
                            state.completed = true;
                        }
                    }
                }

                if state.completed {
                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                        return;
                    }
                }
            }
        });

        Self { state }
    }
}

impl<T> Future for RecvFuture<T> {
    type Output = Option<std::result::Result<T, FutError<T>>>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let (state, _cvar) = &*self.state;
        let mut state = state.lock().unwrap();

        if state.completed {
            Poll::Ready(state.result.take())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<T: IEvent + 'static + Send> IEventReceiver<T> for MpmcReceiver<T> {
    fn recv(&mut self) -> impl Future<Output = Option<Result<T, FutError<T>>>> + Send {
        RecvFuture::new(self.state.take().unwrap())
    }

    fn recv_blocking(&mut self) -> Option<Result<T, FutError<T>>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TestEvent;

    impl IEvent for TestEvent {}
    #[tokio::test]
    async fn test_recv() -> anyhow::Result<()> {
        let (send, mut recv) = create_unbounded_mpm();
        send.send(TestEvent).await?;
        let res = recv.recv().await;

        if let None = res {
            panic!("Returned None")
        }

        let _res = res.unwrap()?;

        Ok(())
    }
}
