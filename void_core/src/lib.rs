use std::future::Future;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Event {}

pub trait Observer<T: Event>: Send {
    fn update(&self, event: &T) -> Result<()>;
}

pub trait Subject: Send {
    type E: Event;
    fn attach(&mut self, observer: impl Observer<Self::E> + 'static);
    fn detach(&mut self, observer: impl Observer<Self::E> + 'static);
    fn notify(&self, event: Self::E) -> Result<()>;
}

pub trait Command {}

pub trait CmdSender<T: Command>: Send {
    fn send(&self, cmd: T) -> impl Future<Output = Result<()>>;
    fn send_blocking(&self, cmd: T) -> Result<()>;
}

pub trait CmdReceiver<T: Command>: Send {
    fn recv(&mut self) -> impl Future<Output = Option<T>> + Send;
    fn recv_blockding(&mut self) -> Option<T>;
}

pub trait System {
    type C: Command;
    fn run(&mut self) -> impl Future<Output = Result<()>> + Send;
    fn run_blocking(&mut self) -> Result<()> {
        Ok(())
    }
}
