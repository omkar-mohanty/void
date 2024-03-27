use core::fmt;
use std::str::FromStr;

use crossbeam_channel::{RecvError, SendError, TryRecvError, TrySendError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FutError<T> {
    TryRecvError(#[from] TryRecvError),
    TrySendError(#[from] TrySendError<T>),
    RecvError(#[from] RecvError),
    SendError(#[from] SendError<T>),
}

impl<T> fmt::Display for FutError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = self.to_string();
        f.write_str(&msg)
    }
}

#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Could not build : {0}")]
    Failed(String),
    #[error("Missing critical field : {0}")]
    Missing(String),
}

#[derive(Error, Debug)]
pub enum SystemError<T> {
    #[error("Could Not recieve event : {0}")]
    RecieveFailure(#[from] FutError<T>),
    #[error("System Failure : {0}")]
    Failed(String),
}
