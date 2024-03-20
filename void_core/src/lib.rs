use std::result::Result;
use thiserror::Error;

pub trait Event: Sized {}

pub trait System: 'static {
    type T: Event + 'static;
    fn process_event(&mut self, event: Self::T) -> Result<(), Error>;
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("System Error `{0}`")]
    System(String),
    #[error("Resource Error `{0}`")]
    Resource(String),
}
