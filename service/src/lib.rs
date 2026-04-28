mod observer;
mod signal;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
}

pub use observer::{Observable, Observer, Resource};
pub use signal::Signal;
