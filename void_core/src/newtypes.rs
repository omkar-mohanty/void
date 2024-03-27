use crate::{FutError, IEvent, IEventReceiver, IEventSender};
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError, TrySendError};
use std::{
    future::Future,
    sync::{Arc, Condvar, Mutex},
    task::{Poll, Waker},
    thread,
};

pub fn create_unbounded_mpmc<T>() -> (MpmcSender<T>, MpmcReceiver<T>) {
    let (send, recv) = unbounded();
    (MpmcSender(send), MpmcReceiver(recv))
}

#[derive(Clone)]
pub struct MpmcReceiver<T>(Receiver<T>);
#[derive(Clone)]
pub struct MpmcSender<T>(Sender<T>);

struct InnerStateSend<T> {
    pub sender: Sender<T>,
    pub msg: Option<T>,
    pub result: Option<std::result::Result<(), FutError<T>>>,
    pub waker: Option<Waker>,
    pub completed: bool,
}

struct SendFuture<T> {
    state: Arc<(Mutex<InnerStateSend<T>>, Condvar)>,
}

impl<T: Send + 'static> SendFuture<T> {
    pub fn new(sender: Sender<T>, msg: T) -> Self {
        let state = InnerStateSend {
            sender,
            msg: Some(msg),
            result: None,
            waker: None,
            completed: false,
        };

        let state = Arc::new((Mutex::new(state), Condvar::new()));
        let thread_state = Arc::clone(&state);

        thread::spawn(move || {
            let (state, cvar) = &*thread_state;

            let mut state_mut = state.lock().unwrap();

            // Wait until poll function sets the waker.
            if state_mut.waker.is_none() {
                state_mut = cvar.wait(state_mut).unwrap();
            }

            drop(state_mut);

            loop {
                let mut state = state.lock().unwrap();

                if let Some(waker) = state.waker.take() {
                    waker.wake();
                }

                if state.completed {
                    return;
                }

                state = cvar.wait(state).unwrap();
                let msg = state.msg.take().unwrap();
                let res = state.sender.try_send(msg);

                match res {
                    Ok(event) => {
                        state.result = Some(Ok(event));
                        state.completed = true;
                    }
                    Err(err) => match err {
                        TrySendError::Full(msg) => state.msg = Some(msg),
                        TrySendError::Disconnected(msg) => {
                            state.msg = Some(msg);
                            state.completed = true;
                        }
                    },
                }
            }
        });

        Self { state }
    }
}

impl<T> Future for SendFuture<T> {
    type Output = Result<(), FutError<T>>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let (state, cvar) = &*self.state;
        let mut state = state.lock().unwrap();
        cvar.notify_one();
        if state.completed {
            Poll::Ready(state.result.take().unwrap())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<T: IEvent + 'static + Send> IEventSender<T> for MpmcSender<T> {
    fn send(&self, cmd: T) -> impl Future<Output = Result<(), FutError<T>>> {
        SendFuture::new(self.0.clone(), cmd)
    }

    fn send_blocking(&self, cmd: T) -> Result<(), FutError<T>> {
        self.0.send(cmd)?;
        Ok(())
    }
}

struct InnerStateRecv<T> {
    pub recv: Receiver<T>,
    pub result: Option<std::result::Result<T, FutError<T>>>,
    pub waker: Option<Waker>,
    pub completed: bool,
}

struct RecvFuture<T> {
    state: Arc<(Mutex<InnerStateRecv<T>>, Condvar)>,
}

impl<T: Send + 'static> RecvFuture<T> {
    pub fn new(recv: Receiver<T>) -> Self {
        let state = InnerStateRecv {
            recv,
            result: None,
            waker: None,
            completed: false,
        };

        let state = Arc::new((Mutex::new(state), Condvar::new()));

        let thread_state = Arc::clone(&state);

        std::thread::spawn(move || {
            let (state, cvar) = &*thread_state;

            let mut state_mut = state.lock().unwrap();

            // Wait until poll function sets the waker.
            if state_mut.waker.is_none() {
                state_mut = cvar.wait(state_mut).unwrap();
            }

            drop(state_mut);

            loop {
                let mut state = state.lock().unwrap();

                if let Some(waker) = state.waker.take() {
                    waker.wake();
                }

                if state.completed {
                    return;
                }

                state = cvar.wait(state).unwrap();

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
        let (state, cvar) = &*self.state;
        let mut state = state.lock().unwrap();
        cvar.notify_one();
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
        RecvFuture::new(self.0.clone())
    }

    fn recv_blocking(&mut self) -> Option<Result<T, FutError<T>>> {
        match self.0.recv() {
            Ok(event) => Some(Ok(event)),
            Err(msg) => Some(Err(FutError::from(msg))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TestEvent;

    impl IEvent for TestEvent {}

    #[tokio::test]
    async fn test_send_recv() -> anyhow::Result<()> {
        let (send, mut recv) = create_unbounded_mpmc();
        send.send(TestEvent).await?;
        let res = recv.recv().await;

        if let None = res {
            panic!("Returned None")
        }

        let _res = res.unwrap()?;

        Ok(())
    }
}
